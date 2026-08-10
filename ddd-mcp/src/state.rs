//! Serve-time shared state — LSP hosts plus same-session declarations.
//!
//! One [`ServeState`] lives for the whole `ddd serve` process. The host
//! manager holds the per-language children; the session log records seam
//! plus pattern declarations filed *through this process*, which is what
//! "a matching declaration in the same session" means in PRD §8.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ddd_lsp::HostManager;
use serde_json::Value;

/// One declaration filed during this serve session.
#[derive(Debug, Clone)]
pub struct Declared {
    /// The entry id (`seam/...` or `pat/...`).
    pub id: String,
    /// Where the declaration says the contract lives.
    pub contract_location: String,
    /// The symbol the declaration names, when it names one.
    pub symbol: Option<String>,
}

#[derive(Default)]
pub struct SessionLog {
    pub declared: Vec<Declared>,
}

pub struct ServeState {
    pub root: PathBuf,
    pub manager: Mutex<HostManager>,
    pub session: Mutex<SessionLog>,
}

impl ServeState {
    pub fn new(root: PathBuf) -> Self {
        let manager = Mutex::new(HostManager::new(root.clone()));
        Self { root, manager, session: Mutex::new(SessionLog::default()) }
    }

    /// Record a declaration for same-session matching.
    pub fn record_declaration(&self, id: &str, contract_location: &str, symbol: Option<String>) {
        if let Ok(mut log) = self.session.lock() {
            log.declared.push(Declared {
                id: id.to_string(),
                contract_location: contract_location.to_string(),
                symbol,
            });
        }
    }

    /// Resolve a tool `file` argument against the repo root.
    pub fn resolve_file(&self, rel_or_abs: &str) -> PathBuf {
        let p = Path::new(rel_or_abs);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.root.join(p)
        }
    }

    /// A path repo-relative for display plus event rows.
    pub fn rel_display(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

/// Required string argument, trimmed, non-empty.
pub fn req_str(args: &Value, key: &str) -> Result<String, String> {
    opt_str(args, key).ok_or_else(|| format!("missing required argument `{key}`"))
}

/// Optional string argument (absent, null, or blank → `None`).
pub fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Optional string argument with no normalisation at all.
///
/// [`opt_str`] trims and treats blank as absent, which is right for names,
/// ids, and prose. It is wrong for anything whose value *is* the payload:
/// `apply_edit`'s `new_text` is a whole file, and trimming it silently
/// deletes the trailing newline plus any leading blank line, on every
/// language. Callers holding content reach for this instead.
pub fn raw_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

/// Optional integer argument.
pub fn opt_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}
