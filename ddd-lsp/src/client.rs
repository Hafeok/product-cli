//! LSP client core — a stdio child process with request correlation.
//!
//! One reader thread routes responses to waiting callers by id, answers
//! server-to-client requests with safe defaults, records notifications
//! into [`ServerState`] (readiness flags, restore notes, published
//! diagnostics). Callers block on a channel with a deadline, so a hung
//! host surfaces as a `Timeout`, never as a silent hang (PRD §11).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::BufReader;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use crate::error::{LspError, Result};
use crate::jsonrpc;

/// What the reader thread has learned from the host so far.
#[derive(Default)]
pub struct ServerState {
    /// Notification methods seen at least once (readiness signals live here).
    pub flags: Mutex<BTreeSet<String>>,
    /// The most recent params per notification method. Hosts that repeat one
    /// notification and carry its meaning in the payload — rust-analyzer's
    /// `experimental/serverStatus` arrives first with `quiescent: false` —
    /// cannot be read off `flags` alone. Bounded by the number of distinct
    /// methods a host sends, not by how often it sends them.
    pub last_params: Mutex<BTreeMap<String, Value>>,
    /// Human-readable observations (restore requests, log lines worth keeping).
    pub notes: Mutex<Vec<String>>,
    /// Latest `textDocument/publishDiagnostics` params per uri.
    pub diagnostics: Mutex<BTreeMap<String, Value>>,
}

type Pending = Arc<Mutex<HashMap<i64, mpsc::Sender<Value>>>>;

/// A live language-server child spoken to over stdio.
pub struct LspClient {
    child: Mutex<Child>,
    writer: Arc<Mutex<ChildStdin>>,
    pending: Pending,
    pub state: Arc<ServerState>,
    next_id: AtomicI64,
}

impl LspClient {
    /// Spawn `command` in `cwd` with piped stdio; stderr is discarded.
    pub fn spawn(command: &[String], cwd: &Path) -> Result<Self> {
        let (program, args) = command.split_first().ok_or_else(|| {
            LspError::Spawn("empty host command".to_string())
        })?;
        let mut child = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LspError::Spawn(format!("{program}: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| LspError::Spawn("no stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| LspError::Spawn("no stdout".into()))?;
        let writer = Arc::new(Mutex::new(stdin));
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
        let state = Arc::new(ServerState::default());
        let (w, p, s) = (writer.clone(), pending.clone(), state.clone());
        std::thread::spawn(move || reader_loop(BufReader::new(stdout), w, p, s));
        Ok(Self { child: Mutex::new(child), writer, pending, state, next_id: AtomicI64::new(1) })
    }

    /// Send a request; block for the result up to `timeout`.
    pub fn request(&self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        if let Ok(mut pending) = self.pending.lock() {
            pending.insert(id, tx);
        }
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        self.write(&msg)?;
        let response = rx.recv_timeout(timeout).map_err(|_| {
            if let Ok(mut pending) = self.pending.lock() {
                pending.remove(&id);
            }
            LspError::Timeout(format!("{method} unanswered after {timeout:?}"))
        })?;
        if let Some(err) = response.get("error") {
            return Err(LspError::Rpc {
                code: err.get("code").and_then(Value::as_i64).unwrap_or(0),
                message: err.get("message").and_then(Value::as_str).unwrap_or("?").to_string(),
            });
        }
        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Send a notification (no answer expected).
    pub fn notify(&self, method: &str, params: Value) -> Result<()> {
        self.write(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
    }

    fn write(&self, msg: &Value) -> Result<()> {
        let mut w = self.writer.lock().map_err(|_| LspError::Io("writer poisoned".into()))?;
        jsonrpc::write_message(&mut *w, msg).map_err(LspError::from)
    }

    /// Is the child process still running?
    pub fn is_alive(&self) -> bool {
        match self.child.lock() {
            Ok(mut child) => matches!(child.try_wait(), Ok(None)),
            Err(_) => false,
        }
    }

    /// The child's OS pid (for external health tooling plus tests).
    pub fn pid(&self) -> Option<u32> {
        self.child.lock().ok().map(|c| c.id())
    }

    /// Best-effort teardown: kill the child; the reader thread ends on EOF.
    pub fn shutdown(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Route every inbound frame: responses to callers, server requests to
/// [`answer_server_request`], notifications into the shared state.
fn reader_loop(
    mut r: BufReader<std::process::ChildStdout>,
    writer: Arc<Mutex<ChildStdin>>,
    pending: Pending,
    state: Arc<ServerState>,
) {
    while let Ok(Some(msg)) = jsonrpc::read_message(&mut r) {
        let has_id = msg.get("id").map(|v| !v.is_null()).unwrap_or(false);
        let method = msg.get("method").and_then(Value::as_str).map(str::to_string);
        match (has_id, method) {
            (true, Some(m)) => answer_server_request(&writer, &msg, &m),
            (true, None) => {
                let id = msg.get("id").and_then(Value::as_i64).unwrap_or(-1);
                let tx = pending.lock().ok().and_then(|mut p| p.remove(&id));
                if let Some(tx) = tx {
                    let _ = tx.send(msg);
                }
            }
            (false, Some(m)) => record_notification(&state, &m, &msg),
            (false, None) => {}
        }
    }
}

/// Answer a server-to-client request with the safest possible default, so
/// hosts that expect an editor keep running under a headless client.
fn answer_server_request(writer: &Arc<Mutex<ChildStdin>>, msg: &Value, method: &str) {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let response = match method {
        "workspace/configuration" => {
            let n = msg
                .pointer("/params/items")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(1);
            json!({"jsonrpc": "2.0", "id": id, "result": vec![Value::Null; n]})
        }
        "window/workDoneProgress/create" | "client/registerCapability"
        | "client/unregisterCapability" => {
            json!({"jsonrpc": "2.0", "id": id, "result": Value::Null})
        }
        "workspace/applyEdit" => {
            json!({"jsonrpc": "2.0", "id": id, "result": {"applied": false}})
        }
        _ => json!({"jsonrpc": "2.0", "id": id,
                    "error": {"code": -32601, "message": format!("unhandled: {method}")}}),
    };
    if let Ok(mut w) = writer.lock() {
        let _ = jsonrpc::write_message(&mut *w, &response);
    }
}

/// Keep what matters from a notification: publish-diagnostics per uri, a
/// note for restore-style signals, the method name as a readiness flag.
fn record_notification(state: &ServerState, method: &str, msg: &Value) {
    if method == "textDocument/publishDiagnostics" {
        if let Some(uri) = msg.pointer("/params/uri").and_then(Value::as_str) {
            if let Ok(mut d) = state.diagnostics.lock() {
                d.insert(uri.to_string(), msg.get("params").cloned().unwrap_or(Value::Null));
            }
        }
        return;
    }
    if method.to_ascii_lowercase().contains("restore") {
        if let Ok(mut notes) = state.notes.lock() {
            notes.push(format!("host signalled restore: {method}"));
        }
    }
    if let Ok(mut flags) = state.flags.lock() {
        flags.insert(method.to_string());
    }
    if let Ok(mut last) = state.last_params.lock() {
        last.insert(method.to_string(), msg.get("params").cloned().unwrap_or(Value::Null));
    }
}
