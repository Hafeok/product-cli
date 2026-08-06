//! Detection assembly over a temp repo: walk, routing, joining, ignores.

use std::path::Path;

use super::*;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

fn sarif(driver: &str, results: &str) -> String {
    format!(
        r#"{{"version": "2.1.0", "runs": [{{"tool": {{"driver": {{"name": "{driver}"}}}}, "results": [{results}]}}]}}"#
    )
}

#[test]
fn joins_configured_plus_emitted_under_one_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(
        tmp.path(),
        "src/.editorconfig",
        "[*.cs]\ndotnet_diagnostic.CA2007.severity = warning\n",
    );
    write(
        tmp.path(),
        "diag.sarif",
        &sarif("csc", r#"{"ruleId": "CA2007", "level": "warning"}, {"ruleId": "IDE0051"}"#),
    );
    let state = detect(tmp.path(), None, &[tmp.path().join("diag.sarif")]);
    let both = &state.rules[&("analyzers".into(), "CA2007".into())];
    assert!(both.configured.is_some());
    assert_eq!(both.emitted, Some(("warning".into(), 1, 0)));
    assert!(both.enabled());
    let emitted_only = &state.rules[&("analyzers".into(), "IDE0051".into())];
    assert!(emitted_only.configured.is_none());
    assert!(state.sourced.contains("analyzers"));
    assert!(state.notes.is_empty(), "{:?}", state.notes);
}

#[test]
fn routes_by_driver_then_rule_id_shape() {
    assert_eq!(namespace_for("csc", "CA2007"), "analyzers");
    assert_eq!(namespace_for("bicep", "no-unused-params"), "linter");
    assert_eq!(namespace_for("", "BCP037"), "linter");
    assert_eq!(namespace_for("", "no-unused-params"), "linter");
    assert_eq!(namespace_for("", "clippy::unwrap_used"), "clippy");
    assert_eq!(namespace_for("", "CS8618"), "analyzers");
}

#[test]
fn bicepconfig_is_a_dedicated_source_even_without_rules() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "infra/bicepconfig.json", r#"{"moduleAliases": {}}"#);
    let state = detect(tmp.path(), None, &[]);
    assert!(state.sourced.contains("linter"));
    assert!(!state.sourced.contains("analyzers"));
}

#[test]
fn an_editorconfig_without_diagnostic_lines_is_not_a_source() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), ".editorconfig", "root = true\nindent_size = 4\n");
    let state = detect(tmp.path(), None, &[]);
    assert!(!state.sourced.contains("analyzers"));
}

#[test]
fn cargo_lints_route_to_the_clippy_namespace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "Cargo.toml", "[workspace.lints.clippy]\nunwrap_used = \"deny\"\n");
    let state = detect(tmp.path(), None, &[]);
    let rule = &state.rules[&("clippy".into(), "clippy::unwrap_used".into())];
    assert_eq!(rule.configured.as_ref().map(|c| c.severity.as_str()), Some("deny"));
    assert!(state.sourced.contains("clippy"));
}

#[test]
fn ignore_globs_prune_the_walk() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(
        tmp.path(),
        "fixtures/.editorconfig",
        "[*.cs]\ndotnet_diagnostic.CA2007.severity = warning\n",
    );
    let cfg = DddConfig { ignore: vec!["fixtures/**".into()], ..Default::default() };
    let state = detect(tmp.path(), Some(&cfg), &[]);
    assert!(state.rules.is_empty(), "{:?}", state.rules.keys().collect::<Vec<_>>());
}

#[test]
fn suppressed_emissions_are_counted_separately() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(
        tmp.path(),
        "d.sarif",
        &sarif(
            "csc",
            r#"{"ruleId": "CA2000", "suppressions": [{"kind": "inSource"}]}, {"ruleId": "CA2000"}"#,
        ),
    );
    let state = detect(tmp.path(), None, &[tmp.path().join("d.sarif")]);
    let rule = &state.rules[&("analyzers".into(), "CA2000".into())];
    assert_eq!(rule.emitted, Some(("warning".into(), 2, 1)));
    assert!(rule.source_suppressed());
    assert!(rule.enabled(), "one live emission keeps it enabled");
}

#[test]
fn a_clean_run_still_marks_its_namespace_sourced() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "clean.sarif", &sarif("bicep", ""));
    let state = detect(tmp.path(), None, &[tmp.path().join("clean.sarif")]);
    assert!(state.sourced.contains("linter"));
}

#[test]
fn unreadable_sarif_lands_in_notes_not_a_panic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    write(tmp.path(), "bad.sarif", "not json");
    let state = detect(tmp.path(), None, &[tmp.path().join("bad.sarif")]);
    assert_eq!(state.notes.len(), 1);
    assert!(state.notes[0].contains("bad.sarif"), "{:?}", state.notes);
}

#[test]
fn glob_matching_covers_star_doublestar_and_question() {
    assert!(glob_match("fixtures/**", "fixtures/a/b.cs"));
    assert!(glob_match("fixtures/**", "fixtures"));
    assert!(glob_match("**/*.sarif", "a/b/c.sarif"));
    assert!(glob_match("src/*.cs", "src/a.cs"));
    assert!(!glob_match("src/*.cs", "src/sub/a.cs"));
    assert!(glob_match("a?c/x", "abc/x"));
    assert!(!glob_match("fixtures/**", "src/fixtures.rs"));
}
