//! M6 acceptance: the interception loop over real rust-analyzer.
//!
//! Every policy row is exercised through the interceptor on the fixture
//! crate, once from the side it claims is boundary-forming and once from the
//! neighbouring side it claims is not. Then the full enforce-mode loop —
//! reject, declare, re-apply, correspondence row — on a `pub` item and on a
//! trait-impl addition, which is the row with no `pub` keyword to key on.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// The adapter's own resolved host command as a YAML flow sequence, with the
/// loud failure a missing toolchain component deserves.
fn host_command_yaml() -> String {
    let command = ddd_lsp::adapter::rust::host_command();
    let program = command.first().cloned().unwrap_or_default();
    assert!(
        std::process::Command::new(&program).arg("--version").output().is_ok(),
        "cannot run `{program}`. rust-analyzer is a pinned component of this toolchain \
         (rust-toolchain.toml) — install it with `rustup component add rust-analyzer`."
    );
    serde_json::to_string(&command).unwrap_or_else(|_| "[]".to_string())
}

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ddd-cli/tests/fixtures/rust")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Run `git` in `root`, asserting success.
fn git(root: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// Commit the whole tree, so every governed file's parent state is
/// committed — the precondition for binding (M8 ruling 2).
fn commit_all(root: &Path) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "init"]);
}

/// A governed fixture crate: `.ddd/` scaffold, format-3 config binding the
/// Rust adapter to the real host, the `code` class at `mode` — all
/// committed, because enforce mode only binds against committed parent
/// state (M8).
fn repo(mode: &str) -> (tempfile::TempDir, product_mcp::ToolRegistry) {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd_core::init::apply_init(&ddd_core::init::plan_init(tmp.path())).expect("init");
    std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir");
    std::fs::write(tmp.path().join("Cargo.toml"), fixture("Cargo.toml.txt")).expect("manifest");
    std::fs::write(tmp.path().join("src/lib.rs"), fixture("lib.rs")).expect("lib");
    std::fs::write(tmp.path().join("src/other.rs"), fixture("other.rs")).expect("other");
    std::fs::write(
        tmp.path().join(".ddd/config.yaml"),
        format!(
            "format: 3\nintercept: warn\nignore: []\nintercept_by_class:\n  code: {mode}\nadapter:\n  rust:\n    command: {}\n",
            host_command_yaml()
        ),
    )
    .expect("config");
    git(tmp.path(), &["init", "-q"]);
    git(tmp.path(), &["config", "user.email", "t@example.com"]);
    git(tmp.path(), &["config", "user.name", "T"]);
    commit_all(tmp.path());
    let (_state, registry) = ddd_mcp::serve::build_registry(tmp.path().to_path_buf());
    (tmp, registry)
}

fn call(registry: &product_mcp::ToolRegistry, name: &str, args: Value) -> Value {
    registry.call_tool(name, &args).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn warm(registry: &product_mcp::ToolRegistry) {
    let out = call(registry, "ddd_warmup", json!({"wait_ms": 120_000, "languages": ["rust"]}));
    let host = &out["hosts"][0];
    assert_eq!(host["readiness"]["state"], "ready", "{out}");
}

fn source(root: &Path) -> String {
    std::fs::read_to_string(root.join("src/lib.rs")).expect("read lib.rs")
}

fn apply(registry: &product_mcp::ToolRegistry, new_text: &str) -> Value {
    call(registry, "ddd_apply_edit", json!({"file": "src/lib.rs", "new_text": new_text}))
}

/// The demand raised for `symbol`, or `None` when the edit raised none.
fn demand_for<'a>(result: &'a Value, symbol: &str) -> Option<&'a Value> {
    let list = result.get("demands").or_else(|| result.get("surface"))?.as_array()?;
    list.iter().find(|d| d["surface"]["symbol"] == symbol)
}

fn event_rows(root: &Path) -> Vec<Value> {
    let dir = root.join(".ddd/seams/events");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_else(|_| Vec::new());
    paths.sort();
    paths
        .iter()
        .map(|p| {
            let text = std::fs::read_to_string(p).expect("row");
            serde_yaml::from_str::<Value>(&text).expect("yaml")
        })
        .collect()
}

// ------------------------------------------------------- the row sweep

/// Warn mode applies every edit and reports the classification, so one host
/// can walk the whole table. Each case names the row it must select.
#[test]
fn every_policy_row_triggers_on_the_fixture_and_its_neighbour_does_not() {
    let (tmp, registry) = repo("warn");
    warm(&registry);
    let base = source(tmp.path());

    // Each case: (description, the edit, symbol, expected row, expected surface).
    let cases: Vec<(&str, String, &str, &str, bool)> = vec![
        (
            "a new pub fn",
            base.replace("pub fn describe(", "pub fn added(v: u32) -> u32 { v }\n\npub fn describe("),
            "added",
            "rs-add-exposed",
            true,
        ),
        (
            "a new private fn",
            base.replace("fn internal_helper(", "fn also_private(v: u32) -> u32 { v }\n\nfn internal_helper("),
            "also_private",
            "rs-private-nonsurface",
            false,
        ),
        (
            "a signature change on a pub fn",
            base.replace("pub fn describe(colour: &Colour)", "pub fn describe(colour: &Colour, terse: bool)"),
            "describe",
            "rs-signature-exposed",
            true,
        ),
        (
            "a signature change on a private fn",
            base.replace("fn internal_helper(value: u32)", "fn internal_helper(value: u64)"),
            "internal_helper",
            "rs-private-nonsurface",
            false,
        ),
        (
            "removing a pub fn",
            base.replace(
                "pub fn describe(colour: &Colour) -> &'static str {\n    match colour {\n        Colour::Red => \"red\",\n        Colour::Green => \"green\",\n    }\n}\n",
                "",
            ),
            "describe",
            "rs-remove-exposed",
            true,
        ),
        (
            "demoting a pub fn",
            base.replace("pub fn describe(", "fn describe("),
            "describe",
            "rs-visibility-boundary",
            true,
        ),
        (
            "a new member on a public trait",
            base.replace(
                "pub trait Greeter {\n    fn greet(&self) -> String;",
                "pub trait Greeter {\n    fn greet(&self) -> String;\n    fn wave(&self) -> String;",
            ),
            "wave",
            "rs-trait-member",
            true,
        ),
        (
            "a new member on a private trait",
            base.replace(
                "trait Internal {\n    fn tally(&self) -> u32;",
                "trait Internal {\n    fn tally(&self) -> u32;\n    fn recount(&self) -> u32;",
            ),
            "recount",
            "rs-private-nonsurface",
            false,
        ),
        (
            "a supertrait bound on a public trait",
            base.replace("pub trait Greeter {", "pub trait Greeter: Clone {"),
            "Greeter",
            "rs-trait-definition",
            true,
        ),
        (
            "a new variant on a public enum",
            base.replace("pub enum Colour {\n    Red,", "pub enum Colour {\n    Red,\n    Blue,"),
            "Blue",
            "rs-add-exposed",
            true,
        ),
        (
            "a new variant on a private enum",
            base.replace("enum Mode {\n    _On,", "enum Mode {\n    _On,\n    _Idle,"),
            "_Idle",
            "rs-private-nonsurface",
            false,
        ),
        (
            "a derive change on a public struct",
            base.replace("#[derive(Clone)]\npub struct Person", "#[derive(Clone, Debug)]\npub struct Person"),
            "Person",
            "rs-derive-change",
            true,
        ),
        (
            "a pub(crate) signature change, default posture",
            base.replace("pub(crate) fn age(&self) -> u32", "pub(crate) fn age(&self) -> u64"),
            "age",
            "rs-crate-visible-nonsurface",
            false,
        ),
        (
            "a new trait impl on a public type",
            base.replace(
                "pub enum Colour {",
                "impl std::fmt::Debug for Person {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name) }\n}\n\npub enum Colour {",
            ),
            "impl std::fmt::Debug for Person",
            "rs-trait-impl",
            true,
        ),
        (
            "a new inherent impl on a public type",
            base.replace(
                "pub enum Colour {",
                "impl Colour {\n    fn shade(&self) -> u32 { 0 }\n}\n\npub enum Colour {",
            ),
            "impl Colour",
            "rs-add-exposed",
            true,
        ),
        (
            "an impl whose trait and self type are both declared elsewhere",
            base.replace(
                "pub enum Colour {",
                "impl other::Remote for other::Far {\n    fn ping(&self) -> u32 { 0 }\n}\n\npub enum Colour {",
            ),
            "impl other::Remote for other::Far",
            "rs-trait-impl-unresolved",
            true,
        ),
    ];

    let mut covered: Vec<&str> = Vec::new();
    for (what, edited, symbol, expected_row, expected_surface) in &cases {
        assert_ne!(&edited[..], &base[..], "the edit for `{what}` changed nothing");
        let result = apply(&registry, edited);
        assert_eq!(result["status"], "applied", "{what}: {result}");
        match demand_for(&result, symbol) {
            Some(d) => {
                assert!(*expected_surface, "{what}: expected no demand, got {d}");
                assert_eq!(d["surface"]["rule"], json!(expected_row), "{what}: {d}");
            }
            None => assert!(
                !*expected_surface,
                "{what}: expected row {expected_row} on `{symbol}`, got {result}"
            ),
        }
        covered.push(expected_row);
        // Put the fixture back for the next case.
        apply(&registry, &base);
    }

    for row in [
        "rs-add-exposed",
        "rs-signature-exposed",
        "rs-remove-exposed",
        "rs-visibility-boundary",
        "rs-trait-member",
        "rs-trait-definition",
        "rs-derive-change",
        "rs-trait-impl",
        "rs-trait-impl-unresolved",
        "rs-crate-visible-nonsurface",
        "rs-private-nonsurface",
    ] {
        assert!(covered.contains(&row), "no case demonstrated {row}");
    }
}

#[test]
fn the_crate_visible_row_flips_with_the_library_posture() {
    let (tmp, registry) = repo("warn");
    std::fs::write(
        tmp.path().join(".ddd/config.yaml"),
        format!(
            "format: 3\nintercept: warn\nignore: []\nintercept_by_class:\n  code: warn\nadapter:\n  rust:\n    command: {}\n    internal_is_surface: true\n",
            host_command_yaml()
        ),
    )
    .expect("config");
    let (_state, registry2) = ddd_mcp::serve::build_registry(tmp.path().to_path_buf());
    drop(registry);
    warm(&registry2);
    let base = source(tmp.path());
    let edited = base.replace("pub(crate) fn age(&self) -> u32", "pub(crate) fn age(&self) -> u64");
    let result = apply(&registry2, &edited);
    let d = demand_for(&result, "age").unwrap_or_else(|| panic!("no demand: {result}"));
    assert_eq!(d["surface"]["rule"], json!("rs-crate-visible-surface"));
    assert_eq!(d["surface"]["visibility"], json!("pub(crate)"));
}

// -------------------------------------------------- the interception loop

/// Reject → declare → re-apply → correspondence row, on a `pub` item.
#[test]
fn a_pub_item_is_rejected_until_a_seam_is_declared() {
    let (tmp, registry) = repo("enforce");
    warm(&registry);
    let base = source(tmp.path());
    let edited =
        base.replace("pub fn describe(", "pub fn added(v: u32) -> u32 { v }\n\npub fn describe(");

    let rejected = apply(&registry, &edited);
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    assert_eq!(source(tmp.path()), base, "a rejected edit must not reach the disk");
    let demand = demand_for(&rejected, "added").unwrap_or_else(|| panic!("{rejected}"));
    assert_eq!(demand["surface"]["kind"], json!("fn"));
    assert_eq!(demand["surface"]["visibility"], json!("public"));
    assert_eq!(demand["surface"]["rule"], json!("rs-add-exposed"));
    assert_eq!(demand["surface"]["language"], json!("rust"));
    // Facts pre-filled, judgment blank (dec/ddd/rejection-facts-prefilled).
    assert_eq!(demand["template"]["verdict_knowledge"], json!(""));
    assert!(demand["template"]["signature"].is_null() || demand["surface"]["signature"].is_string());
    // The template carries the pre-computed binding for this transition.
    let binding = demand["template"]["binding"].clone();
    assert_eq!(binding["symbol"], json!("added"), "{demand}");
    assert_eq!(binding["file"], json!("src/lib.rs"), "{demand}");
    assert!(binding["before"].is_string() && binding["after"].is_string(), "{demand}");
    assert!(binding["base_revision"].is_string(), "{demand}");

    call(
        &registry,
        "ddd_declare_seam",
        json!({
            "id": "seam/rust/added",
            "boundary": "fn added in src/lib.rs",
            "contract_location": "src/lib.rs#added",
            "symbol": "added",
            "verdict_knowledge": "callers learn that a u32 maps to a u32 with no failure mode, so no error path is owed on this side",
            "binding": binding,
        }),
    );

    let applied = apply(&registry, &edited);
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"], json!(["seam/rust/added"]));
    assert!(source(tmp.path()).contains("pub fn added"));

    let rows = event_rows(tmp.path());
    let rejected_row = rows
        .iter()
        .find(|r| r["symbol"] == "added" && r["outcome"] == "rejected")
        .unwrap_or_else(|| panic!("no rejected row in {rows:?}"));
    assert_eq!(rejected_row["language"], json!("rust"));
    assert_eq!(rejected_row["kind"], json!("fn"));
    assert_eq!(rejected_row["visibility"], json!("public"));
    assert_eq!(rejected_row["change"], json!("added"));
    assert_eq!(rejected_row["rule"], json!("rs-add-exposed"));
    assert!(rejected_row["signature"].as_str().unwrap_or_default().contains("fn added"));
    assert!(rejected_row["reference_count"].is_number(), "{rejected_row}");

    let linked_row = rows
        .iter()
        .find(|r| r["symbol"] == "added" && r["outcome"] == "applied-linked")
        .unwrap_or_else(|| panic!("no linked row in {rows:?}"));
    assert_eq!(linked_row["linked_declaration"], json!("seam/rust/added"));

    // The declaration itself carries the machine-authored structural facts.
    let seam: Value = serde_yaml::from_str(
        &std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-rust-added.yaml"))
            .expect("declaration"),
    )
    .expect("yaml");
    assert_eq!(seam["metadata"]["kind"], json!("fn"));
    assert_eq!(seam["metadata"]["visibility"], json!("public"));
}

/// The same loop for a trait impl, which carries no visibility keyword at
/// all — the row exists because nothing in the syntax marks it as surface.
#[test]
fn a_trait_impl_addition_is_rejected_until_a_seam_is_declared() {
    let (tmp, registry) = repo("enforce");
    warm(&registry);
    let base = source(tmp.path());
    let edited = base.replace(
        "pub enum Colour {",
        "impl std::fmt::Debug for Person {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.name) }\n}\n\npub enum Colour {",
    );

    let rejected = apply(&registry, &edited);
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let demand = demand_for(&rejected, "impl std::fmt::Debug for Person")
        .unwrap_or_else(|| panic!("{rejected}"));
    assert_eq!(demand["surface"]["kind"], json!("trait-impl"));
    assert_eq!(demand["surface"]["rule"], json!("rs-trait-impl"));
    // The two halves of the participation ride along on the demand.
    assert_eq!(demand["surface"]["extra"]["trait"], json!("Debug"));
    assert_eq!(demand["surface"]["extra"]["self_type"], json!("Person"));

    call(
        &registry,
        "ddd_declare_seam",
        json!({
            "id": "seam/rust/person-debug",
            "boundary": "impl std::fmt::Debug for Person in src/lib.rs",
            "contract_location": "src/lib.rs#impl std::fmt::Debug for Person",
            "symbol": "impl std::fmt::Debug for Person",
            "verdict_knowledge": "callers learn that Person may be formatted for diagnostics, which commits the name field to appearing in logs and makes it a disclosure surface rather than an internal field",
            "binding": demand["template"]["binding"],
        }),
    );

    let applied = apply(&registry, &edited);
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"], json!(["seam/rust/person-debug"]));

    let rows = event_rows(tmp.path());
    let row = rows
        .iter()
        .find(|r| r["symbol"] == "impl std::fmt::Debug for Person" && r["outcome"] == "applied-linked")
        .unwrap_or_else(|| panic!("no linked row in {rows:?}"));
    assert_eq!(row["kind"], json!("trait-impl"));
    assert_eq!(row["extra"]["self_type"], json!("Person"));
}

#[test]
fn a_private_edit_needs_no_declaration_even_under_enforce() {
    let (tmp, registry) = repo("enforce");
    warm(&registry);
    let base = source(tmp.path());
    let edited = base.replace(
        "fn internal_helper(",
        "fn also_private(v: u32) -> u32 { v }\n\nfn internal_helper(",
    );
    let result = apply(&registry, &edited);
    assert_eq!(result["status"], "applied", "{result}");
    assert_eq!(result["surface_events"], json!(0));
    assert!(source(tmp.path()).contains("fn also_private"));
}
