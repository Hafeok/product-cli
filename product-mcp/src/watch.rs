//! Filesystem watcher feeding the SSE change channel.
//!
//! Watches the `.product/` source tree and ticks a `tokio::broadcast` on every
//! create/modify/remove — so the live view refreshes regardless of who mutated
//! the graph (MCP write tools, the CLI, a hand edit, or a `git checkout`). The
//! watcher runs on its own OS thread and bridges `notify`'s sync callback into
//! the async broadcast every SSE subscriber listens on.

use std::path::PathBuf;

use tokio::sync::broadcast;

/// A `()` tick emitted whenever a watched file changes.
pub type ChangeTx = broadcast::Sender<()>;

/// Spawn a recursive watcher over `dir`. Returns the broadcast sender; the
/// returned sender is kept alive by the server's shared state, and each SSE
/// connection subscribes a fresh receiver. Watcher-setup failure is non-fatal:
/// the channel still exists, the view just falls back to manual refresh.
pub fn spawn(dir: PathBuf) -> ChangeTx {
    let (tx, _keepalive) = broadcast::channel(16);
    let ticks = tx.clone();
    std::thread::spawn(move || run(dir, ticks));
    tx
}

fn run(dir: PathBuf, ticks: ChangeTx) {
    use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let (raw_tx, raw_rx) = std::sync::mpsc::channel();
    let mut watcher = match RecommendedWatcher::new(move |res| { let _ = raw_tx.send(res); }, Config::default()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("  view: file watcher unavailable ({e}); live refresh disabled");
            return;
        }
    };
    if let Err(e) = watcher.watch(&dir, RecursiveMode::Recursive) {
        eprintln!("  view: cannot watch {} ({e}); live refresh disabled", dir.display());
        return;
    }
    for res in raw_rx {
        let Ok(event) = res else { continue };
        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
            // A send error only means no SSE clients are currently subscribed.
            let _ = ticks.send(());
        }
    }
}
