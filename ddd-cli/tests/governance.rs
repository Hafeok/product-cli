//! `ddd diff` scenarios over the committed fixtures (M2 acceptance, PRD §10).
//!
//! Each language gets the three finding kinds twice: a violating pass and a
//! governed (clean) pass. Tests run against the committed SARIF only; the
//! `#[ignore]`d tests at the bottom regenerate SARIF from real builds.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn ddd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ddd").expect("binary");
    cmd.current_dir(dir);
    cmd
}

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(rel)
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

fn copy_fixture(root: &Path, rel: &str) {
    let content = std::fs::read_to_string(fixture(rel)).expect("fixture");
    write(root, rel.rsplit('/').next().expect("name"), &content);
}

fn claim_yaml(id: &str) -> String {
    format!(
        "format: 1\nid: {id}\nstatement: the rule set pays rent\nstatus: reported\nevidence: exercised here\nfalsifier: a counterexample\nowner: none\nchanged: 2026-08-01\n"
    )
}

fn decision_yaml(id: &str, claim: &str) -> String {
    format!(
        "format: 1\nid: {id}\ntitle: Adopt\nrationale: it pays rent\nprincipal: Emil\nbased_on: [{claim}]\n"
    )
}

fn risk_yaml(id: &str) -> String {
    format!(
        "format: 1\nid: {id}\nkind: risk-acceptance\ntitle: Accepted\nrationale: bounded risk\nprincipal: Emil\n"
    )
}

/// A repo with the C# fixture config in place plus an initialised store.
fn csharp_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    copy_fixture(tmp.path(), "csharp/.editorconfig");
    copy_fixture(tmp.path(), "csharp/Program.cs");
    copy_fixture(tmp.path(), "csharp/minimal.csproj");
    tmp
}

fn bicep_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    copy_fixture(tmp.path(), "bicep/bicepconfig.json");
    copy_fixture(tmp.path(), "bicep/main.bicep");
    tmp
}

fn csharp_sarif() -> String {
    fixture("csharp/diag.sarif").to_string_lossy().into_owned()
}

fn bicep_sarif() -> String {
    fixture("bicep/bicep.sarif").to_string_lossy().into_owned()
}

#[test]
fn csharp_ungoverned_stale_and_uncited_are_all_found() {
    let tmp = csharp_repo();
    // One STALE seed: an entry for a rule no source knows.
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA9998\n    severity: warning\n    governance: UNGOVERNED\n",
    );
    ddd(tmp.path())
        .args(["diff", "--sarif", &csharp_sarif()])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("UNGOVERNED analyzers/CA2007")
                .and(predicate::str::contains("UNGOVERNED analyzers/IDE0051"))
                .and(predicate::str::contains("emitted source only"))
                .and(predicate::str::contains("STALE analyzers/CA9998"))
                .and(predicate::str::contains("UNCITED_SUPPRESSION analyzers/CA1303"))
                .and(predicate::str::contains("UNCITED_SUPPRESSION analyzers/CA2000"))
                .and(predicate::str::contains("suppressed in source")),
        )
        .stderr(predicate::str::contains("governance escape(s)"));
}

#[test]
fn a_governed_csharp_repo_diffs_clean_and_why_resolves_the_chain() {
    let tmp = csharp_repo();
    write(tmp.path(), ".ddd/claims/DDD-cs-1.yaml", &claim_yaml("DDD-cs-1"));
    write(tmp.path(), ".ddd/decisions/adopt.yaml", &decision_yaml("dec/cs/adopt", "DDD-cs-1"));
    write(tmp.path(), ".ddd/decisions/risk-loc.yaml", &risk_yaml("risk/cs/loc"));
    write(tmp.path(), ".ddd/decisions/risk-dispose.yaml", &risk_yaml("risk/cs/dispose"));
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/cs/adopt\n  - rule_id: IDE0051\n    severity: warning\n    decision: dec/cs/adopt\n  - rule_id: CA1303\n    severity: none\n    decision: dec/cs/adopt\n    suppression:\n      risk_acceptance: risk/cs/loc\n  - rule_id: CA2000\n    severity: warning\n    decision: dec/cs/adopt\n    suppression:\n      risk_acceptance: risk/cs/dispose\n",
    );
    ddd(tmp.path()).arg("validate").assert().success();
    ddd(tmp.path())
        .args(["diff", "--sarif", &csharp_sarif()])
        .assert()
        .success()
        .stdout(predicate::str::contains("governance diff clean"));
    // The M2 acceptance thread: rule id -> decision -> principal -> claim.
    ddd(tmp.path())
        .args(["why", "CA2007"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("governed by:")
                .and(predicate::str::contains("principal: Emil"))
                .and(predicate::str::contains("claim DDD-cs-1 [reported]")),
        );
}

/// dec/ddd/why-resolves-three-ways: a detected rule with no manifest entry
/// is an escape, not an unknown id, and must not read as "not found".
#[test]
fn why_separates_a_detected_unfiled_rule_from_an_unknown_id() {
    let tmp = csharp_repo();
    // CA2007 is configured in .editorconfig; nothing maps it yet.
    ddd(tmp.path())
        .args(["why", "CA2007"])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("detected, not in the manifest")
                .and(predicate::str::contains(".editorconfig"))
                .and(predicate::str::contains("UNGOVERNED")),
        )
        .stderr(predicate::str::contains("resolves to no decision"));

    // An id no source knows still reports as a genuine miss.
    ddd(tmp.path())
        .args(["why", "CA9999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// The escape report names what it could not check rather than printing a
/// bare `clean` (dec/ddd/report-coverage-explicit).
#[test]
fn escapes_report_states_its_own_coverage() {
    let tmp = csharp_repo();
    write(tmp.path(), ".ddd/claims/DDD-cs-1.yaml", &claim_yaml("DDD-cs-1"));
    // A format-1 decision: its basis edge carries no pin to compare against.
    write(tmp.path(), ".ddd/decisions/adopt.yaml", &decision_yaml("dec/cs/adopt", "DDD-cs-1"));
    ddd(tmp.path())
        .args(["report", "escapes", "--sarif", &csharp_sarif()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("clean — 0 pinned basedOn edge(s) checked")
                .and(predicate::str::contains("not checkable: 1 unpinned edge(s) — dec/cs/adopt"))
                .and(predicate::str::contains("not checkable: 1 live claim(s) carry no revalidate_by")),
        );
}

#[test]
fn bicep_ungoverned_stale_and_uncited_are_all_found() {
    let tmp = bicep_repo();
    write(
        tmp.path(),
        ".ddd/manifest/linter.yaml",
        "format: 1\nrules:\n  - rule_id: use-recent-api-versions\n    severity: warning\n    governance: UNGOVERNED\n",
    );
    ddd(tmp.path())
        .args(["diff", "--sarif", &bicep_sarif()])
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("UNGOVERNED linter/no-unused-params")
                .and(predicate::str::contains("STALE linter/use-recent-api-versions"))
                .and(predicate::str::contains("UNCITED_SUPPRESSION linter/no-hardcoded-env-urls"))
                .and(predicate::str::contains("suppressed in config")),
        );
}

#[test]
fn a_governed_bicep_repo_diffs_clean() {
    let tmp = bicep_repo();
    write(tmp.path(), ".ddd/claims/DDD-iac-1.yaml", &claim_yaml("DDD-iac-1"));
    write(tmp.path(), ".ddd/decisions/adopt.yaml", &decision_yaml("dec/iac/adopt", "DDD-iac-1"));
    write(tmp.path(), ".ddd/decisions/risk-urls.yaml", &risk_yaml("risk/iac/urls"));
    write(
        tmp.path(),
        ".ddd/manifest/linter.yaml",
        "format: 1\nrules:\n  - rule_id: no-unused-params\n    severity: warning\n    decision: dec/iac/adopt\n  - rule_id: no-hardcoded-env-urls\n    severity: \"off\"\n    decision: dec/iac/adopt\n    suppression:\n      risk_acceptance: risk/iac/urls\n",
    );
    ddd(tmp.path())
        .args(["diff", "--sarif", &bicep_sarif()])
        .assert()
        .success()
        .stdout(predicate::str::contains("governance diff clean"));
}

#[test]
fn severity_thresholds_in_config_downgrade_findings() {
    let tmp = csharp_repo();
    write(
        tmp.path(),
        ".ddd/config.yaml",
        "format: 2\nintercept: warn\nignore: []\ndiff:\n  ungoverned: warn\n  uncited_suppression: warn\n",
    );
    ddd(tmp.path())
        .args(["diff", "--sarif", &csharp_sarif()])
        .assert()
        .success()
        .stdout(predicate::str::contains("UNGOVERNED").and(predicate::str::contains("(warning)")));
}

#[test]
fn without_a_source_staleness_is_a_note_not_a_finding() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    governance: UNGOVERNED\n",
    );
    ddd(tmp.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("no detection source seen")
                .and(predicate::str::contains("STALE").not()),
        );
}

#[test]
fn ignore_globs_exclude_config_from_detection() {
    let tmp = csharp_repo();
    write(
        tmp.path(),
        ".ddd/config.yaml",
        "format: 1\nintercept: warn\nignore:\n  - \".editorconfig\"\n",
    );
    ddd(tmp.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("governance diff clean"));
}

#[test]
fn report_escapes_names_a_seeded_basis_loss_with_both_statuses() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    // The decision pinned the claim as reported@2026-08-02; the claim has
    // since been retired — the report must name decision, claim, both.
    write(
        tmp.path(),
        ".ddd/claims/DDD-cs-1.yaml",
        "format: 1\nid: DDD-cs-1\nstatement: s\nstatus: retired\nevidence: killed by a counterexample repo\nowner: none\nchanged: 2026-08-05\n",
    );
    write(
        tmp.path(),
        ".ddd/decisions/adopt.yaml",
        "format: 2\nid: dec/cs/adopt\ntitle: Adopt\nrationale: r\nprincipal: Emil\nbased_on:\n  - claim: DDD-cs-1\n    status: reported\n    changed: 2026-08-02\n",
    );
    ddd(tmp.path())
        .args(["report", "escapes", "--today", "2026-08-06"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                "decision dec/cs/adopt — basedOn DDD-cs-1 pinned reported@2026-08-02, now retired@2026-08-05",
            )
            .and(predicate::str::contains("escape(s)")),
        );
}

#[test]
fn report_escapes_flags_claims_past_their_cadence() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(
        tmp.path(),
        ".ddd/claims/DDD-cs-1.yaml",
        "format: 2\nid: DDD-cs-1\nstatement: s\nstatus: reported\nevidence: e\nfalsifier: f\nowner: none\nchanged: 2026-01-01\nrevalidate_by: 2026-06-01\n",
    );
    ddd(tmp.path())
        .args(["report", "escapes", "--today", "2026-08-06"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "claim DDD-cs-1 [reported] — revalidate_by 2026-06-01 has passed",
        ));
    ddd(tmp.path())
        .args(["report", "escapes", "--today", "2026-05-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no escaped decisions"));
}

#[test]
fn report_escapes_carries_the_diff_findings_as_its_first_section() {
    let tmp = csharp_repo();
    ddd(tmp.path())
        .args(["report", "escapes", "--sarif", &csharp_sarif(), "--today", "2026-08-06"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("== governance diff ==")
                .and(predicate::str::contains("UNGOVERNED analyzers/CA2007")),
        );
}

/// Local-only regeneration of the committed C# SARIF (needs the .NET SDK
/// plus network for restore). Verifies the documented invocation works and
/// the output feeds the same ingestion path.
#[test]
#[ignore = "needs a local .NET SDK; committed SARIF covers CI"]
fn regenerate_csharp_sarif_from_a_real_build() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copy_fixture(tmp.path(), "csharp/minimal.csproj");
    copy_fixture(tmp.path(), "csharp/Program.cs");
    copy_fixture(tmp.path(), "csharp/.editorconfig");
    let status = std::process::Command::new("dotnet")
        .args(["build", "minimal.csproj", "-p:ErrorLog=diag.sarif%2Cversion=2.1"])
        .current_dir(tmp.path())
        .status()
        .expect("dotnet on PATH");
    assert!(status.success(), "dotnet build failed");
    let text = std::fs::read_to_string(tmp.path().join("diag.sarif")).expect("sarif written");
    let runs = ddd_core::sarif::parse(&text).expect("regenerated SARIF ingests");
    assert!(!runs.is_empty());
}

/// Local-only regeneration of the committed Bicep SARIF (needs the Bicep CLI).
#[test]
#[ignore = "needs a local Bicep CLI; committed SARIF covers CI"]
fn regenerate_bicep_sarif_from_a_real_lint() {
    let tmp = tempfile::tempdir().expect("tempdir");
    copy_fixture(tmp.path(), "bicep/main.bicep");
    copy_fixture(tmp.path(), "bicep/bicepconfig.json");
    let output = std::process::Command::new("bicep")
        .args(["lint", "main.bicep", "--diagnostics-format", "sarif"])
        .current_dir(tmp.path())
        .output()
        .expect("bicep on PATH");
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let runs = ddd_core::sarif::parse(&text).expect("regenerated SARIF ingests");
    assert_eq!(runs.len(), 1);
    assert!(runs[0].results.iter().any(|r| r.rule_id == "no-unused-params"), "{text}");
}

// ------------------------------------------------------------- M7 webpair

/// A repo carrying the committed webpair fixtures: the pair on disk, both
/// web-tool configs at the root, their JSON outputs plus the token file
/// registered in `detect` (format 4).
fn webpair_repo() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd_core::init::apply_init(&ddd_core::init::plan_init(tmp.path())).expect("init");
    for rel in ["webpair/page.html", "webpair/style.css"] {
        let content = std::fs::read_to_string(fixture(rel)).expect("fixture");
        write(tmp.path(), rel, &content);
    }
    write(
        tmp.path(),
        ".stylelintrc.json",
        &std::fs::read_to_string(fixture("webpair/.stylelintrc.json")).expect("fixture"),
    );
    write(
        tmp.path(),
        ".htmlvalidate.json",
        &std::fs::read_to_string(fixture("webpair/.htmlvalidate.json")).expect("fixture"),
    );
    for rel in ["webpair/stylelint.out.json", "webpair/htmlvalidate.out.json"] {
        let content = std::fs::read_to_string(fixture(rel)).expect("fixture");
        write(tmp.path(), rel, &content);
    }
    write(
        tmp.path(),
        ".ddd/config.yaml",
        "format: 4\nintercept: warn\nignore: []\ndetect:\n  stylelint: [\"webpair/stylelint.out.json\"]\n  htmlvalidate: [\"webpair/htmlvalidate.out.json\"]\n  tokens: [\"webpair/style.css\"]\npair:\n  units:\n    - html: [\"webpair/*.html\"]\n      css: [\"webpair/style.css\"]\n  ignore_classes: [\"js-*\"]\n",
    );
    tmp
}

/// Manifests governing one stylelint rule, one html-validate rule, and the
/// token file's custom properties, each mapping to a decision.
fn govern_webpair(root: &Path) {
    write(root, ".ddd/claims/DDD-web-t.yaml", &claim_yaml("DDD-web-t"));
    write(root, ".ddd/decisions/tokens.yaml", &decision_yaml("dec/web/token-discipline", "DDD-web-t"));
    write(root, ".ddd/decisions/markup.yaml", &decision_yaml("dec/web/markup-validity", "DDD-web-t"));
    write(
        root,
        ".ddd/manifest/stylelint.yaml",
        "format: 1\nrules:\n  - rule_id: declaration-property-value-disallowed-list\n    severity: error\n    decision: dec/web/token-discipline\n",
    );
    write(
        root,
        ".ddd/manifest/htmlvalidate.yaml",
        "format: 1\nrules:\n  - rule_id: no-dup-class\n    severity: error\n    decision: dec/web/markup-validity\n",
    );
    write(
        root,
        ".ddd/manifest/tokens.yaml",
        "format: 1\nrules:\n  - rule_id: \"--color-ok\"\n    severity: color\n    decision: dec/web/token-discipline\n  - rule_id: \"--gap\"\n    severity: dimension\n    decision: dec/web/token-discipline\n",
    );
}

/// The M2 join, live for the web namespaces: configured + emitted rules
/// with no manifest entry are UNGOVERNED escapes.
#[test]
fn webpair_ungoverned_rules_and_tokens_are_found() {
    let tmp = webpair_repo();
    ddd(tmp.path())
        .arg("diff")
        .assert()
        .failure()
        .stdout(
            predicate::str::contains("UNGOVERNED stylelint/declaration-property-value-disallowed-list")
                .and(predicate::str::contains("UNGOVERNED htmlvalidate/no-dup-class"))
                .and(predicate::str::contains("UNGOVERNED tokens/--color-ok")),
        );
}

/// Governed: diff clean, and `why` resolves a stylelint rule id and a
/// design token to their decisions through the same manifest path.
#[test]
fn webpair_governed_diffs_clean_and_why_resolves_rule_and_token() {
    let tmp = webpair_repo();
    govern_webpair(tmp.path());
    ddd(tmp.path()).arg("validate").assert().success();
    ddd(tmp.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("governance diff clean"));
    ddd(tmp.path())
        .args(["why", "declaration-property-value-disallowed-list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("governed by:")
                .and(predicate::str::contains("dec/web/token-discipline")),
        );
    // The token file is a manifest of design decisions: a token resolves.
    ddd(tmp.path())
        .args(["why", "--", "--color-ok"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("severity color")
                .and(predicate::str::contains("dec/web/token-discipline")),
        );
}

/// Both class-contract directions in `report escapes`: an orphan class and
/// a dead selector are findings, not folklore; the check states coverage.
#[test]
fn webpair_report_escapes_carries_both_contract_directions() {
    let tmp = webpair_repo();
    govern_webpair(tmp.path());
    ddd(tmp.path())
        .args(["report", "escapes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("== html+css pair contract ==").and(
            predicate::str::contains("clean — 1 unit(s)"),
        ));
    // Break both directions: consume an undefined class, strand a selector.
    let html = std::fs::read_to_string(tmp.path().join("webpair/page.html")).expect("read");
    write(
        tmp.path(),
        "webpair/page.html",
        &html.replace("class=\"btn\"", "class=\"btn btn-primary\""),
    );
    let css = std::fs::read_to_string(tmp.path().join("webpair/style.css")).expect("read");
    write(tmp.path(), "webpair/style.css", &format!("{css}.ghost {{ color: red; }}\n"));
    ddd(tmp.path())
        .args(["report", "escapes"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("orphan-class `btn-primary`")
                .and(predicate::str::contains("dead-selector `.ghost`")),
        );
}
