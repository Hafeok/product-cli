//! Host lifecycle against the mock LSP binary: readiness gating, request
//! correlation, crash recovery (kill -9), diagnostics capture.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ddd_lsp::adapter;
use ddd_lsp::host::{Host, Readiness};
use ddd_lsp::protocol::flatten_symbols;

fn mock_command(extra: &[&str]) -> Vec<String> {
    let mut cmd = vec![env!("CARGO_BIN_EXE_ddd-mock-lsp").to_string()];
    cmd.extend(extra.iter().map(|s| s.to_string()));
    cmd
}

fn write(dir: &Path, rel: &str, content: &str) -> PathBuf {
    let path = dir.join(rel);
    std::fs::write(&path, content).expect("write fixture");
    path
}

const CS: &str = "public class Api\n{\n    public void Ping() { }\n}\n";

/// A csharp-adapter host wired to the mock; `roslyn` turns the handshake on.
fn host(dir: &Path, roslyn: bool) -> Host {
    let adapter = adapter::for_language("csharp").expect("adapter");
    let extra: &[&str] = if roslyn { &["--handshake", "roslyn"] } else { &[] };
    Host::new(adapter, mock_command(extra), dir.to_path_buf())
}

#[test]
fn answers_document_symbols_when_ready() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let file = write(tmp.path(), "Api.cs", CS);
    let mut h = host(tmp.path(), true);
    let ready = h.wait_ready(Duration::from_secs(10)).expect("readiness");
    assert!(matches!(ready, Readiness::Ready { .. }), "{ready:?}");
    h.ensure_open(&file).expect("open");
    let result = h
        .request(
            "textDocument/documentSymbol",
            serde_json::json!({"textDocument": {"uri": ddd_lsp::protocol::to_uri(&file)}}),
        )
        .expect("symbols");
    let flat = flatten_symbols(&result);
    assert!(flat.iter().any(|s| s.name == "Ping"), "{flat:?}");
}

#[test]
fn loading_window_is_reported_not_hung() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    std::env::set_var("DDD_MOCK_READY_DELAY_MS", "600");
    let mut h = host(tmp.path(), true);
    let first = h.readiness().expect("probe");
    std::env::remove_var("DDD_MOCK_READY_DELAY_MS");
    match first {
        Readiness::Loading { detail, .. } => {
            assert!(detail.contains("loading"), "{detail}");
        }
        other => panic!("expected loading, got {other:?}"),
    }
    let later = h.wait_ready(Duration::from_secs(10)).expect("readiness");
    assert!(matches!(later, Readiness::Ready { .. }), "{later:?}");
    // The mock raised a restore-style notification during load; the host
    // keeps it as a note (PRD §5.1 restore quirk).
    assert!(h.notes().iter().any(|n| n.contains("restore")), "{:?}", h.notes());
}

#[test]
fn kill_dash_nine_recovers_on_the_next_call() {
    let tmp = tempfile::tempdir().expect("tmp");
    write(tmp.path(), "Api.csproj", "<Project/>");
    let file = write(tmp.path(), "Api.cs", CS);
    let mut h = host(tmp.path(), false);
    h.ensure_open(&file).expect("open");
    let pid = h.pid().expect("pid");
    let status = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .expect("kill");
    assert!(status.success());
    std::thread::sleep(Duration::from_millis(200));
    // Next call must respawn, re-open the document, and answer.
    h.ensure_open(&file).expect("re-open after crash");
    let result = h
        .request(
            "textDocument/documentSymbol",
            serde_json::json!({"textDocument": {"uri": ddd_lsp::protocol::to_uri(&file)}}),
        )
        .expect("symbols after restart");
    assert!(!flatten_symbols(&result).is_empty());
    assert_eq!(h.restarts, 1);
    assert_ne!(h.pid(), Some(pid));
}

#[test]
fn published_diagnostics_are_captured_from_a_sidecar() {
    let tmp = tempfile::tempdir().expect("tmp");
    let file = write(tmp.path(), "main.bicep", "param unusedParam string = 'x'\n");
    write(
        tmp.path(),
        "main.bicep.diag.json",
        r#"[{"range": {"start": {"line": 0, "character": 6}, "end": {"line": 0, "character": 17}},
             "severity": 2, "code": "no-unused-params", "source": "bicep", "message": "unused"}]"#,
    );
    let adapter = adapter::for_language("bicep").expect("adapter");
    let mut h = Host::new(adapter, mock_command(&[]), tmp.path().to_path_buf());
    h.ensure_open(&file).expect("open");
    let mut published = None;
    for _ in 0..50 {
        published = h.published_diagnostics(&file).expect("state");
        if published.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let params = published.expect("diagnostics published");
    assert_eq!(
        params.pointer("/diagnostics/0/code").and_then(|v| v.as_str()),
        Some("no-unused-params")
    );
}
