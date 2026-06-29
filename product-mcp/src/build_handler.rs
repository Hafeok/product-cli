//! `product_build_run` handler — run a deliverable build over MCP (§5).
//!
//! Build's orchestration is CLI-coupled (worker dispatch, the LSP/verify gates,
//! progress printing), so rather than re-host it here we reuse the binary: the
//! running `product` process hosts this MCP server, so `current_exe` *is*
//! `product`. We spawn `<exe> build <deliverable> …`, block until it finishes,
//! then parse the persisted `.product/build/<id>.session.json` as the report.

use std::path::Path;
use std::process::Command;

use product_core::pf::build_metrics::BuildSession;
use serde_json::{json, Value};

/// Run a build for the deliverable named in `args`, returning its report.
pub fn run(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let deliverable = args
        .get("deliverable")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "missing required arg: deliverable".to_string())?;

    let exe = std::env::current_exe().map_err(|e| format!("cannot locate product binary: {e}"))?;
    let cli_args = build_args(deliverable, args);

    let output = Command::new(&exe)
        .arg("build")
        .args(&cli_args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to spawn `product build`: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);

    // A dry run never persists a session.json; report the captured plan instead.
    let session = if dry_run { None } else { read_session(repo_root, deliverable) };

    Ok(json!({
        "deliverable": deliverable,
        "ok": output.status.success(),
        "exitCode": output.status.code(),
        "dryRun": dry_run,
        "verdict": session.as_ref().map(|s| &s.verdict),
        "files": session.as_ref().map(|s| &s.files),
        "tokens": session.as_ref().map(|s| json!({
            "total": s.total_tokens(),
            "prompt": s.prompt_tokens(),
            "completion": s.completion_tokens(),
        })),
        "elapsedSecs": session.as_ref().map(|s| s.elapsed_secs),
        "report": stdout,
        "stderr": stderr,
    }))
}

/// Emit the self-contained SPMC prompt for the deliverable (the `--emit-spmc`
/// artifact a Claude Code `-p` session consumes), captured from the CLI's stdout.
pub fn emit(args: &Value, repo_root: &Path) -> Result<Value, String> {
    let deliverable = args
        .get("deliverable")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "missing required arg: deliverable".to_string())?;
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate product binary: {e}"))?;
    // §5.1 — `seam: true` emits the build-seam envelopes instead of the SPMC prompt.
    let seam = args.get("seam").and_then(|v| v.as_bool()).unwrap_or(false);
    let emit_flag = if seam { "--emit-seam" } else { "--emit-spmc" };
    let mut cli = vec![deliverable.to_string(), emit_flag.into(), "--out".into(), "-".into()];
    if let Some(p) = args.get("product").and_then(|v| v.as_str()) {
        cli.push("--product".into());
        cli.push(p.to_string());
    }
    let output = Command::new(&exe)
        .arg("build")
        .args(&cli)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("failed to spawn `product build {emit_flag}`: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if seam {
        // The emitted envelopes are a JSON array; return them parsed.
        let units: Value = serde_json::from_str(stdout.trim())
            .map_err(|e| format!("emitted seam envelopes were not valid JSON: {e}"))?;
        return Ok(json!({ "ok": true, "deliverable": deliverable, "units": units }));
    }
    Ok(json!({
        "ok": true,
        "deliverable": deliverable,
        "spmc": stdout,
    }))
}

/// §5.1 — validate an inbound build-seam verdict event passed inline as `event`.
pub fn verdict(args: &Value, _repo_root: &Path) -> Result<Value, String> {
    let event = args.get("event").ok_or_else(|| "missing required arg: event".to_string())?;
    match product_core::pf::build_seam::validate_verdict(event) {
        Ok(ev) => Ok(json!({
            "ok": true,
            "event_id": ev.event_id,
            "unit_ref": ev.unit_ref,
            "bundle_hash": ev.bundle_hash,
            "verdict": serde_json::to_value(ev.verdict).unwrap_or(Value::Null),
        })),
        Err(e) => Ok(json!({ "ok": false, "error": format!("{e}") })),
    }
}

/// Translate the JSON args into `product build` CLI flags (mirrors the `Build`
/// subcommand: `--lsp` opt-in, `--no-verify` opt-out).
fn build_args(deliverable: &str, args: &Value) -> Vec<String> {
    let mut out = vec![deliverable.to_string()];
    if let Some(role) = args.get("role").and_then(|v| v.as_str()) {
        out.push("--role".into());
        out.push(role.to_string());
    }
    if let Some(jobs) = args.get("jobs").and_then(|v| v.as_u64()) {
        out.push("--jobs".into());
        out.push(jobs.to_string());
    }
    if args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false) {
        out.push("--dry-run".into());
    }
    if args.get("lsp").and_then(|v| v.as_bool()).unwrap_or(false) {
        out.push("--lsp".into());
    }
    if args.get("no_verify").and_then(|v| v.as_bool()).unwrap_or(false) {
        out.push("--no-verify".into());
    }
    if let Some(rounds) = args.get("max_rounds").and_then(|v| v.as_u64()) {
        out.push("--max-rounds".into());
        out.push(rounds.to_string());
    }
    if let Some(budget) = args.get("budget").and_then(|v| v.as_u64()) {
        out.push("--budget".into());
        out.push(budget.to_string());
    }
    if let Some(product) = args.get("product").and_then(|v| v.as_str()) {
        out.push("--product".into());
        out.push(product.to_string());
    }
    out
}

/// Read the build session record the CLI persisted, if present.
fn read_session(repo_root: &Path, deliverable: &str) -> Option<BuildSession> {
    let path = repo_root
        .join(".product")
        .join("build")
        .join(format!("{deliverable}.session.json"));
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}
