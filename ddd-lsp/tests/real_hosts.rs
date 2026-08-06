//! Real-host verification, gated behind `DDD_LSP_E2E=1` — the workspace
//! CI stays hermetic (`dec/ddd/fixtures-not-sdk`); run locally with
//! `roslyn-language-server` plus `bicep-ls` on PATH.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ddd_lsp::adapter;
use ddd_lsp::host::{Host, Readiness};
use ddd_lsp::protocol::{flatten_symbols, to_uri};

fn gated() -> bool {
    std::env::var("DDD_LSP_E2E").ok().as_deref() != Some("1")
}

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ddd-cli/tests/fixtures")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
    let path = root.join(rel);
    std::fs::write(&path, content).expect("write");
    path
}

fn default_host(language: &str, root: &Path) -> Host {
    let a = adapter::for_language(language).expect("adapter");
    let command = a.default_command.iter().map(|s| s.to_string()).collect();
    Host::new(a, command, root.to_path_buf())
}

fn document_symbols(host: &mut Host, file: &Path) -> Vec<String> {
    host.ensure_open(file).expect("open");
    let result = host
        .request("textDocument/documentSymbol", serde_json::json!({"textDocument": {"uri": to_uri(file)}}))
        .expect("symbols");
    flatten_symbols(&result).into_iter().map(|s| s.name).collect()
}

#[test]
fn real_bicep_ls_symbols_and_diagnostics() {
    if gated() {
        return;
    }
    let tmp = tempfile::tempdir().expect("tmp");
    let file = write(tmp.path(), "storage-module.bicep", &fixture("bicep/storage-module.bicep"));
    let mut host = default_host("bicep", tmp.path());
    let ready = host.wait_ready(Duration::from_secs(60)).expect("readiness");
    assert!(matches!(ready, Readiness::Ready { .. }), "{ready:?}");
    let names = document_symbols(&mut host, &file);
    assert!(names.iter().any(|n| n.contains("accountName")), "{names:?}");
    // deployedAt is deliberately unused — the linter should say so.
    let mut published = None;
    for _ in 0..100 {
        published = host.published_diagnostics(&file).expect("state");
        if published.is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let params = published.expect("published diagnostics");
    let text = params.to_string();
    assert!(text.contains("no-unused-params"), "{text}");
}

#[test]
fn real_roslyn_reaches_ready_and_answers_symbols() {
    if gated() {
        return;
    }
    let tmp = tempfile::tempdir().expect("tmp");
    write(
        tmp.path(),
        "Api.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>",
    );
    let file = write(tmp.path(), "Library.cs", &fixture("csharp/Library.cs"));
    let mut host = default_host("csharp", tmp.path());
    let ready = host.wait_ready(Duration::from_secs(300)).expect("readiness");
    assert!(matches!(ready, Readiness::Ready { .. }), "{ready:?} notes={:?}", host.notes());
    // Roslyn decorates names ("Ping() : void") — assert on prefixes here;
    // the adapter's facts layer strips to bare identifiers.
    let names = document_symbols(&mut host, &file);
    assert!(names.iter().any(|n| n.starts_with("PublicApi")), "{names:?}");
    assert!(names.iter().any(|n| n.starts_with("Ping")), "{names:?}");
}
