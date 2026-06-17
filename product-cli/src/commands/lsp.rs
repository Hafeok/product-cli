//! rust-analyzer session — spawns the language server, drives the protocol.
//!
//! The I/O half of the LSP integration (the pure protocol lives in
//! `product_core::pf::lsp`): a reader thread frames messages off the server's
//! stdout into a channel, while the session writes requests and correlates
//! replies by id, surfacing diagnostics (clippy via the check command), symbols,
//! and references to a coding worker.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use clap::Subcommand;
use product_core::pf::lsp::{self, Diagnostic, Location, Position, Symbol};
use serde_json::Value;

use super::BoxResult;

/// Seconds of silence after diagnostics arrive before we assume flycheck is done.
const IDLE_SECS: u64 = 4;
/// Hard ceiling on a single diagnostics wait (clippy over a cold workspace).
const MAX_SECS: u64 = 120;

#[derive(Subcommand)]
pub enum LspCommands {
    /// Report rust-analyzer diagnostics (clippy + rustc) for a file
    Diagnose {
        /// Path to the source file (relative to the repo root)
        path: String,
    },
    /// Find references to the symbol at a position
    Refs {
        path: String,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        col: u32,
    },
    /// List the document symbols of a file
    Symbols {
        path: String,
    },
}

pub(crate) fn handle_lsp(cmd: LspCommands) -> BoxResult {
    let root = super::shared::domain_root();
    let mut s = LspSession::start(&root)?;
    match cmd {
        LspCommands::Diagnose { path } => {
            let ds = s.diagnostics(&root.join(&path))?;
            print_diagnostics(&path, &ds);
        }
        LspCommands::Symbols { path } => {
            for sym in s.symbols(&root.join(&path))? {
                println!("  {} {} (line {})", sym.kind, sym.name, sym.line + 1);
            }
        }
        LspCommands::Refs { path, line, col } => {
            let locs = s.references(&root.join(&path), Position { line: line.saturating_sub(1), character: col })?;
            for l in &locs {
                println!("  {}:{}:{}", l.path, l.line + 1, l.character + 1);
            }
        }
    }
    s.shutdown();
    Ok(())
}

fn print_diagnostics(path: &str, ds: &[Diagnostic]) {
    if ds.is_empty() {
        println!("{path}: no diagnostics");
        return;
    }
    for d in ds {
        let src = d.source.as_deref().unwrap_or("rustc");
        let code = d.code.as_deref().map(|c| format!(" [{c}]")).unwrap_or_default();
        println!("  {}:{}:{} {} ({src}{code}): {}", path, d.line + 1, d.character + 1, d.severity, d.message);
    }
}

/// A live rust-analyzer process plus the channel its framed messages arrive on.
pub struct LspSession {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    next_id: i64,
    /// Open documents and their last version, so a re-diagnose sends `didChange`
    /// (not a duplicate `didOpen`) and rust-analyzer re-runs flycheck.
    open: std::collections::HashMap<String, i64>,
}

impl LspSession {
    /// Spawn rust-analyzer rooted at `root` and complete the initialize handshake.
    pub fn start(root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut child = Command::new("rust-analyzer")
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to launch rust-analyzer (install with `rustup component add rust-analyzer`): {e}"))?;
        let stdin = child.stdin.take().ok_or("no rust-analyzer stdin")?;
        let stdout = child.stdout.take().ok_or("no rust-analyzer stdout")?;
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(v) = read_frame(&mut reader) {
                if tx.send(v).is_err() {
                    break;
                }
            }
        });
        let mut s = LspSession { child, stdin, rx, next_id: 1, open: std::collections::HashMap::new() };
        let root_uri = lsp::path_to_uri(&root.to_string_lossy());
        let id = s.id();
        s.send(&lsp::initialize_request(id, &root_uri))?;
        s.await_response(id)?;
        s.send(&lsp::initialized_notification())?;
        Ok(s)
    }

    fn id(&mut self) -> i64 {
        let n = self.next_id;
        self.next_id += 1;
        n
    }

    fn send(&mut self, v: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.stdin.write_all(lsp::encode(v).as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Pump messages until the reply with `id` arrives; return its `result`.
    fn await_response(&mut self, id: i64) -> Result<Value, Box<dyn std::error::Error>> {
        let deadline = Instant::now() + Duration::from_secs(MAX_SECS);
        loop {
            let msg = self.recv_until(deadline)?;
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
        }
    }

    fn recv_until(&self, deadline: Instant) -> Result<Value, Box<dyn std::error::Error>> {
        let now = Instant::now();
        let wait = deadline.checked_duration_since(now).ok_or("rust-analyzer timed out")?;
        match self.rx.recv_timeout(wait) {
            Ok(v) => Ok(v),
            Err(RecvTimeoutError::Timeout) => Err("rust-analyzer timed out".into()),
            Err(RecvTimeoutError::Disconnected) => Err("rust-analyzer exited".into()),
        }
    }

    /// Open (or re-sync) + save a file and collect diagnostics once flycheck settles.
    pub fn diagnostics(&mut self, path: &Path) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
        let uri = self.sync_doc(path)?;
        self.send(&lsp::did_save(&uri))?;
        let mut latest: Vec<Diagnostic> = Vec::new();
        let mut seen = false;
        let hard = Instant::now() + Duration::from_secs(MAX_SECS);
        loop {
            match self.rx.recv_timeout(Duration::from_secs(IDLE_SECS)) {
                Ok(msg) => {
                    if is_publish_for(&msg, &uri) {
                        latest = lsp::parse_publish_diagnostics(&msg["params"]);
                        seen = true;
                    }
                    if seen && is_flycheck_end(&msg) {
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) if seen => break,
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() > hard {
                break;
            }
        }
        Ok(latest)
    }

    /// The document symbols of a file.
    pub fn symbols(&mut self, path: &Path) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let uri = self.sync_doc(path)?;
        let id = self.id();
        self.send(&lsp::document_symbol_request(id, &uri))?;
        Ok(lsp::parse_symbols(&self.await_response(id)?))
    }

    /// References to the symbol at `pos` in a file.
    pub fn references(&mut self, path: &Path, pos: Position) -> Result<Vec<Location>, Box<dyn std::error::Error>> {
        let uri = self.sync_doc(path)?;
        let id = self.id();
        self.send(&lsp::references_request(id, &uri, pos))?;
        Ok(lsp::parse_locations(&self.await_response(id)?))
    }

    /// Open the file, or notify a full-content change if already open, bumping the
    /// version so rust-analyzer re-analyses. Returns the document URI.
    fn sync_doc(&mut self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let uri = lsp::path_to_uri(&abs.to_string_lossy());
        let text = std::fs::read_to_string(&abs)?;
        let version = {
            let v = self.open.entry(uri.clone()).or_insert(0);
            *v += 1;
            *v
        };
        if version == 1 {
            self.send(&lsp::did_open(&uri, &text))?;
        } else {
            self.send(&lsp::did_change(&uri, version, &text))?;
        }
        Ok(uri)
    }

    /// Best-effort graceful shutdown.
    pub fn shutdown(&mut self) {
        let id = self.id();
        let _ = self.send(&lsp::shutdown_request(id));
        let _ = self.send(&lsp::exit_notification());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn is_publish_for(msg: &Value, uri: &str) -> bool {
    msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        && msg.get("params").and_then(|p| p.get("uri")).and_then(Value::as_str) == Some(uri)
}

/// True when a `$/progress` `end` marks the cargo check / flycheck pass done.
fn is_flycheck_end(msg: &Value) -> bool {
    if msg.get("method").and_then(Value::as_str) != Some("$/progress") {
        return false;
    }
    let params = msg.get("params");
    let ended = params.and_then(|p| p.get("value")).and_then(|v| v.get("kind")).and_then(Value::as_str) == Some("end");
    let token = params.and_then(|p| p.get("token")).and_then(Value::as_str).unwrap_or("").to_lowercase();
    ended && (token.contains("check") || token.contains("flycheck") || token.contains("clippy"))
}

/// Read one `Content-Length`-framed message off the server stream (None at EOF).
fn read_frame<R: BufRead>(reader: &mut R) -> Option<Value> {
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(n) = lsp::content_length(trimmed) {
            len = n;
        }
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}
