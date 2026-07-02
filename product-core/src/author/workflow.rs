//! What → How → Build session launch.
//!
//! Scaffolds a session's `workflow.json` journal, then launches the agent CLI
//! against the phase-gated `product mcp --workflow` server. The generalisation
//! of [`super::domain`] to the full lifecycle: the server gates the tool
//! surface by phase while every tool writes the canonical `.product` graph
//! directly; `product_session_finalize` validates the graph and closes the
//! session.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::AgentCli;
use crate::error::{ProductError, Result};
use crate::pf::workflow::{Phase, WorkflowSession};

/// Where a session's journal lives under the canonical repo.
pub fn session_root(canonical_root: &Path, session_id: &str) -> PathBuf {
    canonical_root.join(".product").join("sessions").join(session_id)
}

/// The facilitation system prompt for a What→How→Build session.
pub fn render_prompt(product: &str) -> String {
    include_str!("workflow_prompt.md").replace("{{PRODUCT}}", product)
}

/// Create the session journal. Returns the session root. The session works
/// directly on the canonical `.product` graph — there is no workspace copy.
pub fn scaffold(
    canonical_root: &Path,
    session_id: &str,
    product: &str,
    cli: &str,
    until: Phase,
    now: String,
) -> Result<PathBuf> {
    let root = session_root(canonical_root, session_id);
    let session = WorkflowSession::new(session_id, product, cli, until, now);
    session.save(&root)?;
    Ok(root)
}

/// Launch the agent CLI against the phase-gated workflow server for `session_id`.
pub fn launch(session_id: &str, product: &str, cli: AgentCli, canonical_root: &Path) -> Result<()> {
    let prompt = render_prompt(product);
    let tmp = std::env::temp_dir().join(format!(
        "product-session-{}-{}.md",
        session_id,
        chrono::Utc::now().timestamp()
    ));
    std::fs::write(&tmp, &prompt).map_err(|e| ProductError::WriteError {
        path: tmp.clone(),
        message: e.to_string(),
    })?;

    let mcp_json = mcp_config_json(session_id, canonical_root);
    println!("Starting What→How→Build session '{}' for '{}' ({})...", session_id, product, cli);
    println!("  Session: {}", session_root(canonical_root, session_id).display());

    // Bring up the live view, pinned to this session, for the run's duration.
    // Each session picks its own free port so concurrent sessions don't clash.
    // The agent CLI takes over the terminal, so we open the browser ourselves
    // and record the URL (recoverable via `product session show`).
    let root = session_root(canonical_root, session_id);
    let mut view = spawn_view(session_id, canonical_root);
    if let Some((_, port)) = view.as_ref() {
        let url = format!("http://127.0.0.1:{port}/?session={session_id}");
        let _ = std::fs::write(root.join(VIEW_URL_FILE), &url);
        println!("  Live view: {url}");
        // Give the server a moment to bind, then open the browser.
        std::thread::sleep(std::time::Duration::from_millis(500));
        open_browser(&url);
    }
    println!();

    let status = match cli {
        AgentCli::Claude => launch_claude(&tmp, &mcp_json, canonical_root),
        AgentCli::Copilot => launch_copilot(&prompt, &mcp_json, canonical_root),
    };
    if let Some((child, _)) = view.as_mut() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let _ = std::fs::remove_file(root.join(VIEW_URL_FILE)); // the view is gone now
    report(status, cli, &tmp);
    Ok(())
}

/// The file (under the session dir) holding the live view URL while it runs.
pub const VIEW_URL_FILE: &str = "view-url.txt";

/// Open `url` with the system's default browser, best-effort across
/// Linux/WSL/macOS. Hands the URL to the default handler; the first opener that
/// launches wins, silent if none exist.
fn open_browser(url: &str) {
    for opener in ["xdg-open", "open", "wslview"] {
        if Command::new(opener).arg(url).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok() {
            return;
        }
    }
}

/// Ask the OS for a free TCP port on loopback (bind to :0, read the assignment,
/// release it). A small race remains before the child rebinds it — best-effort.
fn pick_free_port() -> Option<u16> {
    std::net::TcpListener::bind(("127.0.0.1", 0)).ok()?.local_addr().ok().map(|a| a.port())
}

/// Spawn the HTTP view server scoped to this session on a free port (best-effort;
/// a clash just means no view). Output is silenced so it does not disturb the
/// agent TUI. Returns the child and the port it was given.
fn spawn_view(session_id: &str, canonical_root: &Path) -> Option<(Child, u16)> {
    let exe = std::env::current_exe().ok()?;
    let port = pick_free_port()?;
    let child = Command::new(exe)
        .args([
            "mcp", "--http", "--workflow",
            "--session", session_id,
            "--repo", &canonical_root.display().to_string(),
            "--port", &port.to_string(),
            "--write",
        ])
        .current_dir(canonical_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    Some((child, port))
}

/// The MCP config pointing the agent at the phase-gated workflow server — this
/// same binary re-invoked as `product mcp --workflow --session <id>`.
fn mcp_config_json(session_id: &str, canonical_root: &Path) -> String {
    let exe = std::env::current_exe().unwrap_or_else(|_| "product".into());
    let args = vec![
        "mcp".to_string(),
        "--workflow".to_string(),
        "--session".to_string(),
        session_id.to_string(),
        "--repo".to_string(),
        canonical_root.display().to_string(),
        "--write".to_string(),
    ];
    let mut servers = serde_json::Map::new();
    servers.insert(
        super::MCP_SERVER_NAME.to_string(),
        serde_json::json!({
            "command": exe.display().to_string(),
            "args": args,
            "cwd": canonical_root.display().to_string()
        }),
    );
    let config = serde_json::json!({ "mcpServers": servers });
    serde_json::to_string(&config).unwrap_or_default()
}

fn launch_claude(prompt_file: &Path, mcp_json: &str, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    let allowed = format!("Read,{}", super::claude_tools_glob());
    Command::new("claude")
        .args([
            "--system-prompt-file", &prompt_file.display().to_string(),
            "--allowedTools", &allowed,
            "--disallowedTools", "Bash,Edit,Write,NotebookEdit",
            "--mcp-config", mcp_json,
            "--strict-mcp-config",
        ])
        .current_dir(root)
        .status()
}

fn launch_copilot(prompt: &str, mcp_json: &str, root: &Path) -> std::io::Result<std::process::ExitStatus> {
    let allowed = format!("read,glob,grep,list,view,{}", super::MCP_SERVER_NAME);
    let allowed = allowed.as_str();
    Command::new("copilot")
        .args([
            "-i", prompt,
            "--additional-mcp-config", mcp_json,
            "--available-tools", allowed,
            "--allow-tool", allowed,
            "--disable-builtin-mcps",
            "--no-custom-instructions",
        ])
        .current_dir(root)
        .status()
}

fn report(status: std::io::Result<std::process::ExitStatus>, cli: AgentCli, prompt_file: &Path) {
    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Session complete. All authored changes are in the canonical spec; `product_session_finalize` validated and closed the session.");
        }
        Ok(s) => eprintln!("Agent exited with status: {}", s),
        Err(e) => {
            eprintln!("Could not start {}: {}", cli, e);
            eprintln!("Ensure '{}' is in your PATH.", cli);
            eprintln!("Facilitation prompt written to: {}", prompt_file.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_mentions_product_and_controls() {
        let p = render_prompt("acme");
        assert!(p.contains("acme"));
        assert!(p.contains("product_workflow_advance"));
        assert!(p.contains("product_session_finalize"));
    }

    #[test]
    fn pick_free_port_returns_a_usable_port() {
        let p = pick_free_port().expect("a free port");
        assert!(p > 0);
        // The port is free right now, so we can bind it ourselves.
        assert!(std::net::TcpListener::bind(("127.0.0.1", p)).is_ok());
    }

    #[test]
    fn scaffold_writes_journal_without_workspace() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product/author-domain/demo")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");
        std::fs::write(root.join(".product/author-domain/demo/demo.ttl"), "# ttl\n").expect("ttl");

        let sr = scaffold(root, "demo-1", "demo", "claude", Phase::Build, "t0".into()).expect("scaffold");
        assert!(sr.join("workflow.json").is_file());
        // Sessions edit canonical directly — no isolated workspace copy.
        assert!(!sr.join("ws").exists());

        let loaded = WorkflowSession::load(&sr).expect("load");
        assert_eq!(loaded.id, "demo-1");
        assert_eq!(loaded.phase, Phase::What);
        assert_eq!(loaded.until, Phase::Build);
    }
}
