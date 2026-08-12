//! M7 acceptance: the full interception loop over the hostless HTML+CSS
//! pair adapter — reject → declare → re-apply → correspondence rows
//! carrying the pair-level facts (both files, direction, token/class id).
//! No language server exists behind any of this; no warmup is called.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ddd-cli/tests/fixtures/webpair")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
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

/// A governed temp repo with the pair map on and the html-css class in the
/// given mode — all committed, because enforce mode only binds against
/// committed parent state (M8).
fn repo(mode: &str) -> (tempfile::TempDir, product_mcp::ToolRegistry) {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd_core::init::apply_init(&ddd_core::init::plan_init(tmp.path())).expect("init");
    write(
        tmp.path(),
        ".ddd/config.yaml",
        &format!(
            "format: 4\nintercept: warn\nintercept_by_class:\n  html-css: {mode}\npair:\n  units:\n    - html: [\"site/*.html\"]\n      css: [\"site/style.css\"]\n  ignore_classes: [\"js-*\"]\n"
        ),
    );
    write(tmp.path(), "site/page.html", &fixture("page.html"));
    write(tmp.path(), "site/style.css", &fixture("style.css"));
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

fn event_rows(root: &Path) -> Vec<Value> {
    let dir = root.join(".ddd/seams/events");
    let mut rows = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let text = std::fs::read_to_string(&p).expect("row");
            rows.push(serde_yaml::from_str::<Value>(&text).expect("yaml"));
        }
    }
    rows
}

#[test]
fn token_change_runs_the_full_loop_with_pair_facts_on_the_row() {
    let (tmp, registry) = repo("enforce");
    let css = fixture("style.css");
    let edited = format!("{css}:root {{ --color-warn: #8a5a00; }}\n");

    // 1. Rejected: a token joined the stylesheet's params undeclared.
    let rejected =
        call(&registry, "ddd_apply_edit", json!({"file": "site/style.css", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let surface = &rejected["demands"][0]["surface"];
    assert_eq!(surface["symbol"], "--color-warn");
    assert_eq!(surface["kind"], "token");
    assert_eq!(surface["rule"], "web-token-membership");
    assert_eq!(surface["language"], "htmlcss");
    assert_eq!(surface["artifact_class"], "html-css");
    assert_eq!(surface["extra"]["pair_direction"], "definition");
    assert_eq!(surface["extra"]["pair_counterpart"], "site/page.html");
    assert!(!std::fs::read_to_string(tmp.path().join("site/style.css"))
        .expect("read")
        .contains("--color-warn"));
    // The template carries the pre-computed binding for this transition.
    let binding = rejected["demands"][0]["template"]["binding"].clone();
    assert_eq!(binding["symbol"], "--color-warn", "{rejected}");
    assert_eq!(binding["file"], "site/style.css", "{rejected}");
    assert!(binding["base_revision"].is_string(), "{rejected}");

    // 2. Declare the seam the demand names, signing the demanded transition.
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/web/color-warn-token",
        "boundary": "design token --color-warn in site/style.css",
        "contract_location": "site/style.css#--color-warn",
        "symbol": "--color-warn",
        "verdict_knowledge": "consumers learn the degraded-state colour without knowing its value; retiring or retyping it breaks every var() site",
        "binding": binding,
    }));

    // 3. Re-apply → applied, linked; the row carries the pair facts.
    let applied =
        call(&registry, "ddd_apply_edit", json!({"file": "site/style.css", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"][0], "seam/web/color-warn-token");
    let rows = event_rows(tmp.path());
    assert_eq!(rows.len(), 2, "{rows:?}");
    let row = &rows[1];
    assert_eq!(row["outcome"], "applied-linked");
    assert_eq!(row["symbol"], "--color-warn");
    assert_eq!(row["kind"], "token");
    assert_eq!(row["language"], "htmlcss");
    assert_eq!(row["artifact_class"], "html-css");
    assert_eq!(row["extra"]["pair_direction"], "definition");
    assert_eq!(row["extra"]["pair_counterpart"], "site/page.html");
    assert!(row.get("reference_count").is_none(), "hostless rows carry no count: {row}");
}

#[test]
fn class_vocabulary_change_runs_the_full_loop() {
    let (tmp, registry) = repo("enforce");
    let css = fixture("style.css");
    let edited = format!("{css}.badge {{ color: var(--color-ok); }}\n");
    let rejected =
        call(&registry, "ddd_apply_edit", json!({"file": "site/style.css", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    assert_eq!(rejected["demands"][0]["surface"]["symbol"], "badge");
    assert_eq!(rejected["demands"][0]["surface"]["rule"], "web-class-membership");
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/web/badge-class",
        "boundary": "class badge in site/style.css",
        "contract_location": "site/style.css#badge",
        "symbol": "badge",
        "verdict_knowledge": "markup learns a status chip exists; removing the class strands every element consuming it",
        "binding": rejected["demands"][0]["template"]["binding"],
    }));
    let applied =
        call(&registry, "ddd_apply_edit", json!({"file": "site/style.css", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    let rows = event_rows(tmp.path());
    let last = rows.last().expect("row");
    assert_eq!(last["symbol"], "badge");
    assert_eq!(last["kind"], "class");
    assert_eq!(last["linked_declaration"], "seam/web/badge-class");
}

#[test]
fn token_value_change_and_markup_edits_apply_untouched() {
    let (tmp, registry) = repo("enforce");
    let css = fixture("style.css");
    let recolored = css.replace("#1a7f37", "#2ea043");
    let out = call(&registry, "ddd_apply_edit",
        json!({"file": "site/style.css", "new_text": recolored}));
    assert_eq!(out["status"], "applied", "{out}");
    assert_eq!(out["surface_events"], 0, "{out}");
    let html = fixture("page.html");
    let restructured = html.replace("<button class=\"btn\">go</button>", "<button class=\"btn\"><span>go</span></button>");
    let out = call(&registry, "ddd_apply_edit",
        json!({"file": "site/page.html", "new_text": restructured}));
    assert_eq!(out["status"], "applied", "{out}");
    assert_eq!(out["surface_events"], 0, "{out}");
    assert!(event_rows(tmp.path()).is_empty());
}

#[test]
fn stylesheet_rewiring_is_demanded_from_the_html_side() {
    let (_tmp, registry) = repo("enforce");
    let html = fixture("page.html");
    let rewired = html.replace("style.css", "other.css");
    let rejected =
        call(&registry, "ddd_apply_edit", json!({"file": "site/page.html", "new_text": rewired}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let symbols: Vec<&str> = rejected["demands"]
        .as_array()
        .expect("demands")
        .iter()
        .map(|d| d["surface"]["symbol"].as_str().expect("symbol"))
        .collect();
    assert!(symbols.contains(&"other.css") && symbols.contains(&"style.css"), "{symbols:?}");
    assert_eq!(rejected["demands"][0]["surface"]["extra"]["pair_direction"], "consumption");
}

#[test]
fn language_tools_say_plainly_there_is_no_host() {
    let (_tmp, registry) = repo("enforce");
    let err = registry
        .call_tool("ddd_find_symbol", &json!({"file": "site/style.css"}))
        .expect_err("no host");
    assert!(err.to_string().contains("no language server"), "{err}");
}
