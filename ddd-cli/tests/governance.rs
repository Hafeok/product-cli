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
