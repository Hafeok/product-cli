//! Language-host lifecycle — lazy start, readiness, crash recovery.
//!
//! A [`Host`] wraps one adapter's child process: spawned on first use,
//! health-checked at every entry (a kill -9'd child is respawned on the
//! next call), never blocking a caller on solution load — while the host
//! is indexing, [`Host::readiness`] says [`Readiness::Loading`] and tools
//! surface that as an explicit loading result (PRD §11).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::adapter::Adapter;
use crate::client::LspClient;
use crate::error::{LspError, Result};
use crate::protocol::to_uri;

/// Default per-request deadline; solution loads take longer, so tools ask
/// readiness first rather than waiting on requests.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Where the host is in its life.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "kebab-case", tag = "state")]
pub enum Readiness {
    /// Not spawned yet (lazy start pending).
    NotStarted,
    /// Alive but still indexing; callers get this instead of a hang.
    Loading { elapsed_ms: u128, detail: String },
    Ready { elapsed_ms: u128 },
}

/// One live (or lazily pending) language server.
pub struct Host {
    adapter: &'static Adapter,
    command: Vec<String>,
    root: PathBuf,
    client: Option<LspClient>,
    started_at: Option<Instant>,
    ready_at: Option<Instant>,
    /// Open documents: sync version plus the text last sent to the host.
    open_docs: HashMap<PathBuf, (i64, String)>,
    /// Restarts performed after crashes (observability for tests + status).
    pub restarts: u32,
    /// `ServerCapabilities` from the last `initialize`, kept because a
    /// capability flag is evidence about the server's *claims* — the
    /// capability probe compares it to what live calls actually do.
    capabilities: Option<Value>,
}

impl Host {
    pub fn new(adapter: &'static Adapter, command: Vec<String>, root: PathBuf) -> Self {
        Self {
            adapter,
            command,
            root,
            client: None,
            started_at: None,
            ready_at: None,
            open_docs: HashMap::new(),
            restarts: 0,
            capabilities: None,
        }
    }

    pub fn language(&self) -> &'static str {
        self.adapter.language
    }

    /// The live client, spawning or respawning as needed. This is the
    /// single entry every call path goes through — crash recovery lives
    /// here and nowhere else.
    fn client(&mut self) -> Result<&LspClient> {
        let dead = self.client.as_ref().map(|c| !c.is_alive()).unwrap_or(false);
        if dead {
            self.client = None;
            self.open_docs.clear();
            self.ready_at = None;
            self.capabilities = None;
            self.restarts += 1;
        }
        if self.client.is_none() {
            let client = LspClient::spawn(&self.command, &self.root)?;
            self.capabilities = Some(initialize(&client, &self.root, self.adapter)?);
            if self.adapter.needs_open_handshake {
                announce_workspace(&client, &self.root)?;
            }
            self.started_at = Some(Instant::now());
            self.client = Some(client);
        }
        self.client.as_ref().ok_or_else(|| LspError::Spawn("client vanished".into()))
    }

    /// Non-blocking readiness probe; spawns lazily on first ask.
    pub fn readiness(&mut self) -> Result<Readiness> {
        let signal = self.adapter.ready;
        let started = {
            let client = self.client()?;
            signal.satisfied(&client.state)
        };
        let since = self.started_at.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        if started {
            if self.ready_at.is_none() {
                self.ready_at = Some(Instant::now());
            }
            Ok(Readiness::Ready { elapsed_ms: since })
        } else {
            Ok(Readiness::Loading {
                elapsed_ms: since,
                detail: format!(
                    "{} is loading the workspace (waiting for {}); retry shortly",
                    self.adapter.language,
                    signal.describe()
                ),
            })
        }
    }

    /// Block until ready or the deadline passes; used by warmup, never by
    /// tool calls.
    pub fn wait_ready(&mut self, deadline: Duration) -> Result<Readiness> {
        let start = Instant::now();
        loop {
            let r = self.readiness()?;
            if matches!(r, Readiness::Ready { .. }) || start.elapsed() > deadline {
                return Ok(r);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Ensure the file is open on the host (didOpen with disk text). The
    /// health probe runs first: after a crash the tracking map is stale
    /// until `client()` clears it, so the check order is load-bearing.
    pub fn ensure_open(&mut self, path: &Path) -> Result<()> {
        self.client()?;
        if self.open_docs.contains_key(path) {
            return Ok(());
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| LspError::Io(format!("{}: {e}", path.display())))?;
        let language_id = self.adapter.language_id;
        let client = self.client()?;
        client.notify(
            "textDocument/didOpen",
            json!({"textDocument": {
                "uri": to_uri(path), "languageId": language_id, "version": 1, "text": text,
            }}),
        )?;
        self.open_docs.insert(path.to_path_buf(), (1, text));
        Ok(())
    }

    /// Ensure the document is open on the host with the supplied text —
    /// the diff path's entry for files that do not exist on disk (deleted
    /// at head, or a committed state the working tree has moved past).
    /// Already-open documents are overlaid instead.
    pub fn open_with_text(&mut self, path: &Path, text: &str) -> Result<()> {
        self.client()?;
        if self.open_docs.contains_key(path) {
            return self.overlay(path, text);
        }
        let language_id = self.adapter.language_id;
        let client = self.client()?;
        client.notify(
            "textDocument/didOpen",
            json!({"textDocument": {
                "uri": to_uri(path), "languageId": language_id, "version": 1, "text": text,
            }}),
        )?;
        self.open_docs.insert(path.to_path_buf(), (1, text.to_string()));
        Ok(())
    }

    /// Replace the file's content on the host (overlay for classification;
    /// the disk is untouched). Sent as one whole-document *range* change:
    /// Roslyn negotiates incremental sync and drops documents on
    /// range-less full-content changes.
    pub fn overlay(&mut self, path: &Path, text: &str) -> Result<()> {
        self.ensure_open(path)?;
        let (version, previous) =
            self.open_docs.get(path).cloned().unwrap_or((1, String::new()));
        let end = end_position(&previous);
        let version = version + 1;
        let client = self.client()?;
        client.notify(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": to_uri(path), "version": version},
                "contentChanges": [{
                    "range": {"start": {"line": 0, "character": 0}, "end": end},
                    "text": text,
                }],
            }),
        )?;
        self.open_docs.insert(path.to_path_buf(), (version, text.to_string()));
        Ok(())
    }

    /// A request against this host with the standard deadline. The two
    /// transient LSP errors — RequestCancelled (-32800) and
    /// ContentModified (-32801), which busy hosts raise mid-index — are
    /// retried briefly instead of surfacing to the tool caller.
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut last = None;
        loop {
            match self.client()?.request(method, params.clone(), REQUEST_TIMEOUT) {
                Err(LspError::Rpc { code, message })
                    if (code == -32800
                        || code == -32801
                        // Roslyn raises this while a freshly opened document
                        // is still being adopted into its project (the
                        // design-time build can take seconds on first open).
                        || message.contains("Document is null"))
                        && Instant::now() < deadline =>
                {
                    last = Some(LspError::Rpc { code, message });
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) if Instant::now() >= deadline => {
                    return Err(last.unwrap_or(e));
                }
                other => return other,
            }
        }
    }

    /// The server's advertised capabilities, spawning lazily if needed.
    ///
    /// Read as a claim, never as a verdict: `capability::probe` calls the
    /// operation before believing the flag.
    pub fn server_capabilities(&mut self) -> Result<Value> {
        self.client()?;
        Ok(self.capabilities.clone().unwrap_or_else(|| json!({})))
    }

    /// Latest published diagnostics for a uri, when the host pushes them.
    pub fn published_diagnostics(&mut self, path: &Path) -> Result<Option<Value>> {
        let uri = to_uri(path);
        let client = self.client()?;
        let map = client.state.diagnostics.lock().map_err(|_| LspError::Io("state".into()))?;
        Ok(map.get(&uri).cloned())
    }

    /// Notes the host has raised (restore signals etc.).
    pub fn notes(&mut self) -> Vec<String> {
        self.client
            .as_ref()
            .and_then(|c| c.state.notes.lock().ok().map(|n| n.clone()))
            .unwrap_or_default()
    }

    /// The child pid, when running (health tooling plus tests).
    pub fn pid(&self) -> Option<u32> {
        self.client.as_ref().and_then(|c| c.pid())
    }
}

/// The position one past the last character of `text` (for the
/// whole-document replacement range).
fn end_position(text: &str) -> Value {
    let lines: Vec<&str> = text.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    json!({"line": last, "character": lines.last().map(|l| l.chars().count()).unwrap_or(0)})
}

/// The `initialize`/`initialized` handshake every LSP host expects, with
/// the adapter's extra capabilities merged over the common set.
fn initialize(client: &LspClient, root: &Path, adapter: &Adapter) -> Result<Value> {
    let root_uri = to_uri(root);
    let mut capabilities = json!({
        "textDocument": {
            "synchronization": {"didSave": true},
            "publishDiagnostics": {},
            "documentSymbol": {"hierarchicalDocumentSymbolSupport": true},
            "diagnostic": {},
            // Declared because it is correct practice, not because it
            // unblocks anything: Roslyn answers `typeHierarchy` without
            // being asked (the under-advertising direction), so its
            // absence was never what stopped the operation.
            "definition": {},
            "typeHierarchy": {"dynamicRegistration": false},
        },
        "workspace": {"workspaceFolders": true, "configuration": true},
        "window": {"workDoneProgress": true},
    });
    merge(&mut capabilities, (adapter.extra_capabilities)());
    let result = client.request(
        "initialize",
        json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "workspaceFolders": [{"uri": root_uri, "name": "workspace"}],
            "capabilities": capabilities,
        }),
        Duration::from_secs(30),
    )?;
    client.notify("initialized", json!({}))?;
    Ok(result.get("capabilities").cloned().unwrap_or_else(|| json!({})))
}

/// Deep-merge `extra` into `base`: objects recurse, everything else
/// overwrites. Keeps an adapter's capability additions from clobbering a
/// sibling key of the common set.
fn merge(base: &mut Value, extra: Value) {
    match (base, extra) {
        (Value::Object(b), Value::Object(e)) => {
            for (k, v) in e {
                merge(b.entry(k).or_insert(Value::Null), v);
            }
        }
        (b, e) => *b = e,
    }
}

/// The Roslyn-style workspace announcement: find the solution (or the
/// projects) and send the community-documented open notifications; the
/// host answers with `workspace/projectInitializationComplete` when the
/// workspace is usable.
fn announce_workspace(client: &LspClient, root: &Path) -> Result<()> {
    let (solutions, projects) = build_inputs(root);
    if let Some(sln) = solutions.first() {
        client.notify("solution/open", json!({"solution": to_uri(sln)}))?;
    } else if !projects.is_empty() {
        let uris: Vec<String> = projects.iter().map(|p| to_uri(p)).collect();
        client.notify("project/open", json!({"projects": uris}))?;
    }
    Ok(())
}

/// Solutions plus projects under the root (shallow-first, pruned walk).
fn build_inputs(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut solutions = Vec::new();
    let mut projects = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if p.is_dir() {
                if !matches!(name.as_str(), ".git" | ".ddd" | "target" | "node_modules" | "bin" | "obj") {
                    stack.push(p);
                }
            } else if name.ends_with(".sln") || name.ends_with(".slnx") {
                solutions.push(p);
            } else if name.ends_with(".csproj") {
                projects.push(p);
            }
        }
    }
    solutions.sort();
    projects.sort();
    (solutions, projects)
}
