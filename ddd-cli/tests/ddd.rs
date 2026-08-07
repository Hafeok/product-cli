//! End-to-end `ddd` CLI scenarios over a temp repo (M1 acceptance, PRD §10).

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn ddd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ddd").expect("binary");
    cmd.current_dir(dir);
    cmd
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

fn claim_yaml(id: &str) -> String {
    format!(
        "format: 1\nid: {id}\nstatement: the arrangement closes the predicate\nstatus: reported\nevidence: exercised on this repo\nfalsifier: a counterexample repo\nowner: none\nchanged: 2026-08-01\n"
    )
}

#[test]
fn init_scaffolds_the_layout_and_is_idempotent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success().stdout(predicate::str::contains("initialised"));
    for dir in ["predicates", "claims", "decisions", "manifest", "patterns", "seams", "shared"] {
        assert!(tmp.path().join(".ddd").join(dir).is_dir(), "missing {dir}");
    }
    assert!(tmp.path().join(".ddd/config.yaml").is_file());
    ddd(tmp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("nothing to create"));
}

#[test]
fn a_fresh_scaffold_validates_conformant() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("conformant"));
}

#[test]
fn validate_without_a_store_exits_nonzero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("run `ddd init`"));
}

#[test]
fn a_statused_predicate_fails_validation_naming_rule_one() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(
        tmp.path(),
        ".ddd/predicates/bad.yaml",
        "predicate:\n  id: pred/x/bad\n  format: 1\n  name: Bad\n  statement: s\n  artifact_class: code\n  ground: []\n  tolerance: t\n  rejection_witness: w\n  status: closed\n",
    );
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("predicates carry no status"));
}

#[test]
fn an_orphan_decision_fails_then_a_claim_repairs_it() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(
        tmp.path(),
        ".ddd/decisions/adopt.yaml",
        "format: 1\nid: dec/x/adopt\ntitle: Adopt\nrationale: because it pays rent\nprincipal: Emil\nbased_on: [DDD-x-1]\n",
    );
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("basedOn claim 'DDD-x-1' does not exist"));
    write(tmp.path(), ".ddd/claims/DDD-x-1.yaml", &claim_yaml("DDD-x-1"));
    ddd(tmp.path()).arg("validate").assert().success();
}

#[test]
fn ungoverned_manifest_entries_warn_without_failing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    governance: UNGOVERNED\n",
    );
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("UNGOVERNED"));
}

#[test]
fn a_suppression_without_a_risk_acceptance_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(tmp.path(), ".ddd/claims/DDD-x-1.yaml", &claim_yaml("DDD-x-1"));
    write(
        tmp.path(),
        ".ddd/decisions/adopt.yaml",
        "format: 1\nid: dec/x/adopt\ntitle: Adopt\nrationale: r\nprincipal: Emil\nbased_on: [DDD-x-1]\n",
    );
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/x/adopt\n    suppression:\n      reason: noisy\n",
    );
    ddd(tmp.path())
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must cite a risk-acceptance"));
}

#[test]
fn why_resolves_a_decision_and_a_bare_diagnostic_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(tmp.path(), ".ddd/claims/DDD-x-1.yaml", &claim_yaml("DDD-x-1"));
    write(
        tmp.path(),
        ".ddd/decisions/adopt.yaml",
        "format: 1\nid: dec/x/adopt\ntitle: Adopt the analyzer\nrationale: because it pays rent\nprincipal: Emil\nbased_on: [DDD-x-1]\ndate: 2026-08-02\n",
    );
    write(
        tmp.path(),
        ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/x/adopt\n",
    );
    ddd(tmp.path())
        .args(["why", "dec/x/adopt"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("principal: Emil")
                .and(predicate::str::contains("claim DDD-x-1 [reported]")),
        );
    ddd(tmp.path())
        .args(["why", "CA2007"])
        .assert()
        .success()
        .stdout(predicate::str::contains("diagnostic analyzers/CA2007").and(predicate::str::contains("governed by:")));
}

#[test]
fn why_on_an_unknown_id_exits_nonzero() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    ddd(tmp.path())
        .args(["why", "dec/x/missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dec/x/missing"));
}

#[test]
fn root_flag_targets_a_store_from_elsewhere() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let elsewhere = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    ddd(elsewhere.path())
        .args(["--root", &tmp.path().to_string_lossy(), "validate"])
        .assert()
        .success();
}

/// `ddd what` over a seeded What graph: the boundaries report, the
/// declaration clears one, and `--strict` is the CI gate.
#[test]
fn what_reports_undeclared_boundaries_and_gates_under_strict() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(tmp.path(), "product.toml", "name = \"acme\"\n");
    // A minimal What graph: one system, one event, one entity.
    write(
        tmp.path(),
        ".product/products/acme/session.json",
        r#"{"product":"acme","graph":{
             "systems":[{"id":"payments-api","label":"Payments","kind":"service"}],
             "events":[{"id":"PaymentAuthorized","label":"Authorized","context":"billing","changes":"Payment"}],
             "entities":[{"id":"Payment","label":"Payment","context":"billing","definition":"a payment in flight"}]},
           "started_at":"2026-08-07T00:00:00Z"}"#,
    );

    // Both boundaries undeclared; the entity is internal elaboration.
    ddd(tmp.path())
        .arg("what")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("UNDECLARED system/payments-api")
                .and(predicate::str::contains("UNDECLARED event/PaymentAuthorized"))
                .and(predicate::str::contains("2 surface element(s): 0 declared, 2 undeclared"))
                .and(predicate::str::contains("1 internal element(s)")),
        );

    // --strict turns it into a gate.
    ddd(tmp.path()).args(["what", "--strict"]).assert().failure();

    // A seam naming the element clears it; the join is `what:<id>`.
    write(
        tmp.path(),
        ".ddd/seams/seam-what-payments-api.yaml",
        "format: 1\nid: seam/what/payments-api\nboundary: the payments system\nverdict_knowledge: callers learn whether a payment is authorized\ncontract_location: what:payments-api\nobligations: []\n",
    );
    ddd(tmp.path())
        .args(["what", "--all"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("declared  system/payments-api — seam/what/payments-api")
                .and(predicate::str::contains("1 declared, 1 undeclared")),
        );
}

#[test]
fn what_says_so_when_there_is_no_product_to_govern() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    ddd(tmp.path())
        .arg("what")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no product"));
}
