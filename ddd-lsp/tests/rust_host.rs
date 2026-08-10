//! Rust host acceptance against real rust-analyzer (`dec/ddd/rust-host-is-real`).
//!
//! Unlike the .NET hosts these are not gated: rust-analyzer is a component of
//! the pinned toolchain, so it is present locally and in CI. A missing
//! component fails loudly rather than skipping — a silent skip is a green
//! exit code standing in for a check that never ran.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ddd_lsp::adapter;
use ddd_lsp::host::{Host, Readiness};
use ddd_lsp::protocol::{flatten_symbols, to_uri, RawSymbol};

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ddd-cli/tests/fixtures/rust")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Materialise the fixture crate into a tempdir as a real cargo package.
pub fn fixture_crate() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir");
    std::fs::write(tmp.path().join("Cargo.toml"), fixture("Cargo.toml.txt")).expect("manifest");
    std::fs::write(tmp.path().join("src/lib.rs"), fixture("lib.rs")).expect("lib");
    std::fs::write(tmp.path().join("src/other.rs"), fixture("other.rs")).expect("other");
    tmp
}

/// The adapter's own resolved host command, with the loud failure a missing
/// toolchain component deserves — a silent skip would be a green exit code
/// standing in for a check that never ran.
fn host_command() -> Vec<String> {
    let command = adapter::rust::host_command();
    let program = command.first().cloned().unwrap_or_default();
    assert!(
        std::process::Command::new(&program).arg("--version").output().is_ok(),
        "cannot run `{program}`. rust-analyzer is a pinned component of this toolchain \
         (rust-toolchain.toml) — install it with `rustup component add rust-analyzer`."
    );
    command
}

fn rust_host(root: &Path) -> Host {
    let a = adapter::for_language("rust").expect("rust adapter registered");
    Host::new(a, host_command(), root.to_path_buf())
}

fn symbols(host: &mut Host, file: &Path) -> Vec<RawSymbol> {
    host.ensure_open(file).expect("open");
    let result = host
        .request("textDocument/documentSymbol", serde_json::json!({"textDocument": {"uri": to_uri(file)}}))
        .expect("documentSymbol");
    flatten_symbols(&result)
}

fn find<'a>(list: &'a [RawSymbol], name: &str) -> &'a RawSymbol {
    list.iter().find(|s| s.name == name).unwrap_or_else(|| {
        panic!("`{name}` not among {:?}", list.iter().map(|s| &s.name).collect::<Vec<_>>())
    })
}

#[test]
fn a_loading_host_says_so_before_it_says_ready() {
    let tmp = fixture_crate();
    let mut host = rust_host(tmp.path());

    // First probe: the server has answered `initialize` but is still
    // fetching sysroot and cargo metadata. It must not claim readiness.
    let first = host.readiness().expect("readiness");
    let Readiness::Loading { detail, .. } = &first else {
        panic!("expected Loading immediately after spawn, got {first:?}");
    };
    assert!(
        detail.contains("experimental/serverStatus"),
        "the loading detail should name what is being waited on: {detail}"
    );

    let ready = host.wait_ready(Duration::from_secs(120)).expect("wait_ready");
    assert!(matches!(ready, Readiness::Ready { .. }), "{ready:?}");
}

#[test]
fn readiness_waits_for_quiescent_not_for_the_first_status_notification() {
    let tmp = fixture_crate();
    let mut host = rust_host(tmp.path());
    let ready = host.wait_ready(Duration::from_secs(120)).expect("wait_ready");
    let Readiness::Ready { elapsed_ms } = ready else {
        panic!("never became ready: {ready:?}");
    };
    // rust-analyzer sends `experimental/serverStatus` with `quiescent: false`
    // within a few hundred milliseconds of `initialize`, and only reports
    // quiescence once indexing has settled. A method-name-only readiness flag
    // would return here almost immediately; this asserts it does not.
    assert!(
        elapsed_ms > 500,
        "became ready in {elapsed_ms}ms — readiness is tracking the notification's arrival, not its payload"
    );
}

#[test]
fn the_symbol_shapes_the_adapter_parses_are_the_ones_the_server_emits() {
    let tmp = fixture_crate();
    let file = tmp.path().join("src/lib.rs");
    let mut host = rust_host(tmp.path());
    host.wait_ready(Duration::from_secs(120)).expect("ready");
    let syms = symbols(&mut host, &file);

    // Traits arrive as LSP `interface`; their members carry container_kind 11.
    assert_eq!(find(&syms, "Greeter").kind, 11);
    assert_eq!(find(&syms, "greet").container_kind, 11);
    // Impl blocks arrive as one `object` symbol named for the whole header —
    // this is where trait participation is read from.
    let trait_impl = find(&syms, "impl Greeter for Person");
    assert_eq!(trait_impl.kind, 19);
    // Structs, enums, variants, modules, consts, type aliases.
    assert_eq!(find(&syms, "Person").kind, 23);
    assert_eq!(find(&syms, "Colour").kind, 10);
    assert_eq!(find(&syms, "Red").kind, 22);
    assert_eq!(find(&syms, "Red").container_kind, 10);
    assert_eq!(find(&syms, "visible").kind, 2);
    assert_eq!(find(&syms, "LIMIT").kind, 14);
    assert_eq!(find(&syms, "Count").kind, 26);
    // And no visibility anywhere in the payload — which is why the adapter
    // slices the source text.
    let payload = serde_json::to_string(&syms).expect("serialise");
    assert!(!payload.contains("pub("), "symbols unexpectedly carry visibility: {payload}");
}

#[test]
fn facts_from_the_real_host_carry_the_graded_visibility() {
    let tmp = fixture_crate();
    let file = tmp.path().join("src/lib.rs");
    let mut host = rust_host(tmp.path());
    host.wait_ready(Duration::from_secs(120)).expect("ready");
    let syms = symbols(&mut host, &file);
    let a = adapter::for_language("rust").expect("adapter");
    let text = std::fs::read_to_string(&file).expect("read");
    let facts = (a.facts)(&text, &syms, &adapter::AdapterFlags::default());

    // `age` exists twice — the private field and the `pub(crate)` accessor —
    // so lookups here carry the container, exactly as `SymbolFacts::key` does.
    let grade = |container: &str, name: &str| {
        facts
            .iter()
            .find(|f| f.name == name && f.container == container)
            .unwrap_or_else(|| panic!("{container}/{name} not in facts"))
            .visibility
            .clone()
    };
    assert_eq!(grade("impl Person", "new"), "public");
    assert_eq!(grade("impl Person", "age"), "pub(crate)");
    assert_eq!(grade("impl Person", "bump"), "private");
    assert_eq!(grade("", "describe"), "public");
    assert_eq!(grade("", "internal_helper"), "private");
    assert_eq!(grade("visible", "exported"), "public");
    // `pub fn` inside a private module is not reachable.
    assert_eq!(grade("capped", "looks_public_is_not"), "private");
    // Struct fields: `pub name` is surface, bare `age` is not, on the same
    // public struct.
    assert_eq!(grade("Person", "name"), "public");
    assert_eq!(grade("Person", "age"), "private");
}

#[test]
fn a_killed_host_is_respawned_on_the_next_call() {
    let tmp = fixture_crate();
    let file = tmp.path().join("src/lib.rs");
    let mut host = rust_host(tmp.path());
    host.wait_ready(Duration::from_secs(120)).expect("ready");
    let before = symbols(&mut host, &file);
    assert!(!before.is_empty());

    let pid = host.pid().expect("pid");
    let status = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status()
        .expect("kill");
    assert!(status.success());
    std::thread::sleep(Duration::from_millis(200));

    // The replacement starts cold, so it is honest about loading again.
    let after_crash = host.readiness().expect("readiness after crash");
    assert!(matches!(after_crash, Readiness::Loading { .. }), "{after_crash:?}");
    assert_eq!(host.restarts, 1);
    assert_ne!(host.pid(), Some(pid));

    host.wait_ready(Duration::from_secs(120)).expect("ready again");
    let after = symbols(&mut host, &file);
    assert_eq!(
        before.iter().map(|s| &s.name).collect::<Vec<_>>(),
        after.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
}
