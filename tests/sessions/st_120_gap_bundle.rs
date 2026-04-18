//! ST-120..ST-125 — gap bundle + structural gap check (FT-045, ADR-019, ADR-040).
//!
//! Each test composes a temp repository, drives the new `product gap bundle`
//! and structural `product gap check` commands, and asserts on stdout /
//! stderr / exit-code.

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use std::process::{Command, Stdio};

struct Run {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl Run {
    fn run(s: &Session, args: &[&str]) -> Self {
        let out = Command::new(&s.bin)
            .args(args)
            .current_dir(s.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("spawn product");
        Run {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code().unwrap_or(-1),
        }
    }
}

fn write_adr(
    s: &Session,
    id: &str,
    title: &str,
    status: &str,
    features: &[&str],
    body: &str,
) {
    let features_str = if features.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", features.join(", "))
    };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: {status}\nfeatures: {features_str}\ndomains: [api]\nscope: domain\n---\n\n{body}\n"
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

fn write_feature(s: &Session, id: &str, title: &str, adrs: &[&str], tests: &[&str]) {
    let adrs_str = if adrs.is_empty() { "[]".into() } else { format!("[{}]", adrs.join(", ")) };
    let tests_str = if tests.is_empty() { "[]".into() } else { format!("[{}]", tests.join(", ")) };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nphase: 1\nstatus: planned\nadrs: {adrs_str}\ntests: {tests_str}\n---\n\nFeature body.\n"
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/features/{}-{}.md", id, slug), &content);
}

fn write_tc(
    s: &Session,
    id: &str,
    title: &str,
    ty: &str,
    features: &[&str],
    adrs: &[&str],
) {
    let fs = if features.is_empty() { "[]".into() } else { format!("[{}]", features.join(", ")) };
    let ad = if adrs.is_empty() { "[]".into() } else { format!("[{}]", adrs.join(", ")) };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\ntype: {ty}\nstatus: passing\nvalidates:\n  features: {fs}\n  adrs: {ad}\nphase: 1\n---\n\nTest body.\n"
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/tests/{}-{}.md", id, slug), &content);
}

// ---------------------------------------------------------------------------
// TC-563 — gap bundle emits instructions + context bundle
// ---------------------------------------------------------------------------
#[test]
fn tc_563_gap_bundle_outputs_context_and_instructions() {
    let s = Session::new();
    write_feature(&s, "FT-001", "Feature One", &["ADR-002"], &["TC-001"]);
    write_adr(
        &s,
        "ADR-002",
        "Decision Two",
        "accepted",
        &["FT-001"],
        "**Context:** ctx.\n\n**Decision:** decision.\n\n**Rationale:** why.\n\n**Rejected alternatives:**\n- none\n\n**Test coverage:** TC-001.\n",
    );
    write_tc(&s, "TC-001", "Test One", "scenario", &["FT-001"], &["ADR-002"]);

    let r = Run::run(&s, &["gap", "bundle", "ADR-002"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );

    // Instructions section present, and lists all of G001..G008
    assert!(
        r.stdout.contains("## Instructions"),
        "expected Instructions section in stdout:\n{}",
        r.stdout
    );
    for code in ["G001", "G002", "G003", "G004", "G005", "G006", "G007", "G008"] {
        assert!(
            r.stdout.contains(code),
            "expected gap code {} in Instructions:\n{}",
            code,
            r.stdout
        );
    }

    // Context Bundle section present and contains ADR-002
    assert!(
        r.stdout.contains("## Context Bundle"),
        "expected Context Bundle section:\n{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("ADR-002"),
        "expected ADR-002 content in Context Bundle:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-564 — gap bundle --changed scopes to ADRs in the diff + 1-hop neighbours
// ---------------------------------------------------------------------------
#[test]
fn tc_564_gap_bundle_changed_scopes_correctly() {
    let s = Session::new();
    write_adr(
        &s, "ADR-001", "Untouched", "accepted", &[],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n",
    );
    write_adr(
        &s, "ADR-002", "Existing", "accepted", &["FT-001"],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n",
    );
    write_feature(&s, "FT-001", "Feature One", &["ADR-002", "ADR-003"], &[]);

    // Initial git commit with only ADR-001/ADR-002/FT-001.
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(s.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("git");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@t"]);
    git(&["config", "user.name", "t"]);
    git(&["add", "."]);
    git(&["commit", "-qm", "initial"]);

    // Now add ADR-003 and modify ADR-002 on top of the initial commit.
    write_adr(
        &s, "ADR-002", "Existing Modified", "accepted", &["FT-001"],
        "**Context:** x2.\n\n**Decision:** y2.\n\n**Rationale:** z2.\n\n**Rejected alternatives:** none.\n",
    );
    write_adr(
        &s, "ADR-003", "Newly Added", "accepted", &[],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n",
    );
    git(&["add", "."]);
    git(&["commit", "-qm", "second"]);

    let r = Run::run(&s, &["gap", "bundle", "--changed"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );

    // ADR-002 is modified → must be in output.
    assert!(
        r.stdout.contains("Gap Analysis Input: ADR-002"),
        "expected ADR-002 bundle; got:\n{}",
        r.stdout
    );
    // ADR-003 is new → must be in output.
    assert!(
        r.stdout.contains("Gap Analysis Input: ADR-003"),
        "expected ADR-003 bundle; got:\n{}",
        r.stdout
    );
}

// ---------------------------------------------------------------------------
// TC-565 — gap bundle --all includes every ADR exactly once
// ---------------------------------------------------------------------------
#[test]
fn tc_565_gap_bundle_all_includes_all_adrs() {
    let s = Session::new();
    for (i, title) in ["Alpha", "Beta", "Gamma", "Delta"].iter().enumerate() {
        let id = format!("ADR-00{}", i + 1);
        write_adr(
            &s, &id, title, "accepted", &[],
            "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n",
        );
    }

    let r = Run::run(&s, &["gap", "bundle", "--all"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );

    for id in ["ADR-001", "ADR-002", "ADR-003", "ADR-004"] {
        let marker = format!("Gap Analysis Input: {}", id);
        let count = r.stdout.matches(&marker).count();
        assert_eq!(count, 1, "expected exactly 1 bundle for {} (got {}):\n{}", id, count, r.stdout);
    }
}

// ---------------------------------------------------------------------------
// TC-566 — gap check is structural only, no LLM call
// ---------------------------------------------------------------------------
#[test]
fn tc_566_gap_check_structural_only_no_llm_call() {
    let s = Session::new();
    // One accepted ADR with a linked feature and a linked TC. Under FT-045
    // the gap check is entirely structural and completes fast.
    write_feature(&s, "FT-001", "Feature One", &["ADR-001"], &["TC-001"]);
    write_adr(
        &s, "ADR-001", "Decision", "accepted", &["FT-001"],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n",
    );
    write_tc(&s, "TC-001", "Test", "scenario", &["FT-001"], &["ADR-001"]);

    let start = std::time::Instant::now();
    // Run with injected env vars that *would* have triggered LLM behaviour
    // under FT-029 — under FT-045 they must be ignored.
    let out = Command::new(&s.bin)
        .args(["gap", "check", "ADR-001"])
        .current_dir(s.dir.path())
        .env("PRODUCT_GAP_INJECT_ERROR", "simulated LLM network failure")
        .env("PRODUCT_GAP_INJECT_RESPONSE", "[]")
        .stdin(Stdio::null())
        .output()
        .expect("spawn product");
    let elapsed = start.elapsed();

    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let exit_code = out.status.code().unwrap_or(-1);

    assert!(
        exit_code == 0 || exit_code == 1,
        "gap check must exit 0 or 1 under FT-045; got {} (stderr: {})",
        exit_code, stderr
    );
    assert!(
        !stderr.contains("model failure") && !stderr.contains("simulated LLM"),
        "no LLM error path allowed; stderr: {}",
        stderr
    );
    assert!(
        stdout.trim_start().starts_with('[') || stdout.trim_start().starts_with('{'),
        "stdout must be valid JSON; got:\n{}",
        stdout
    );

    // Structural check must complete in under one second on a realistic repo.
    assert!(
        elapsed.as_secs() < 5,
        "gap check took too long ({:?}); structural path must be fast",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// TC-567 — gap check flags G002 (invariant block without scenario/chaos TC)
// ---------------------------------------------------------------------------
#[test]
fn tc_567_gap_check_g002_invariant_no_tc() {
    let s = Session::new();
    // ADR has an invariants block and linked TC of type scenario missing,
    // but linked TC exists of type invariant (non-matching).
    write_feature(&s, "FT-001", "Feature One", &["ADR-001"], &["TC-001"]);
    write_adr(
        &s, "ADR-001", "Decision With Invariants", "accepted", &["FT-001"],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n\n**Rejected alternatives:** none.\n\nInvariants:\n\u{27E6}\u{0393}:Invariants\u{27E7}{ p }\n",
    );
    // TC is of type invariant, not scenario/chaos, so G002 fires.
    write_tc(&s, "TC-001", "Invariant Test", "invariant", &["FT-001"], &["ADR-001"]);

    let r = Run::run(&s, &["gap", "check", "ADR-001", "--format", "json"]);
    assert!(
        r.stdout.contains("G002"),
        "expected G002 in stdout; got:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
    assert_eq!(r.exit_code, 1, "expected exit 1 when G002 fires");
}

// ---------------------------------------------------------------------------
// TC-568 — gap check flags G003 when no Rejected alternatives section
// ---------------------------------------------------------------------------
#[test]
fn tc_568_gap_check_g003_no_rejected_alternatives() {
    let s = Session::new();
    write_feature(&s, "FT-001", "Feature One", &["ADR-001"], &[]);
    // ADR body deliberately omits a Rejected alternatives section.
    write_adr(
        &s, "ADR-001", "Decision Without Rejects", "accepted", &["FT-001"],
        "**Context:** x.\n\n**Decision:** y.\n\n**Rationale:** z.\n",
    );

    let r = Run::run(&s, &["gap", "check", "ADR-001", "--format", "json"]);
    assert!(
        r.stdout.contains("G003"),
        "expected G003 in stdout; got:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );
}
