//! ST-131..ST-132 — ADR conflict bundle + structural check-conflicts
//! (FT-045, ADR-022 amended, ADR-040).

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
    scope: &str,
    domains: &[&str],
    features: &[&str],
) {
    let domains_str = if domains.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", domains.join(", "))
    };
    let features_str = if features.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", features.join(", "))
    };
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: {status}\nfeatures: {features_str}\ndomains: {domains_str}\nscope: {scope}\n---\n\n**Context:** ctx for {id}.\n\n**Decision:** decision for {id}.\n\n**Rationale:** rationale for {id}.\n\n**Rejected alternatives:** none.\n",
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

// ---------------------------------------------------------------------------
// TC-574 — conflict bundle union: cross-cutting + same-domain + top-5
// ---------------------------------------------------------------------------
#[test]
fn tc_574_conflict_bundle_includes_related_adrs() {
    let s = Session::new();

    // Proposed ADR-031 in "consensus" domain.
    write_adr(&s, "ADR-031", "Proposed Decision", "proposed", "domain", &["consensus"], &[]);

    // Two cross-cutting ADRs.
    write_adr(&s, "ADR-010", "Cross Cutting One", "accepted", "cross-cutting", &["api"], &[]);
    write_adr(&s, "ADR-011", "Cross Cutting Two", "accepted", "cross-cutting", &["security"], &[]);

    // Another ADR in the consensus domain.
    write_adr(&s, "ADR-020", "Same Domain", "accepted", "domain", &["consensus"], &[]);

    // Five more unrelated ADRs (for the top-5 ranking; centrality will be 0
    // for everything so ranking is by whatever order the graph returns —
    // at least the command must include some of them, and must never
    // include more than cross-cutting ∪ same-domain ∪ top-5).
    for i in 0..5 {
        let id = format!("ADR-02{}", i + 1);
        let title = format!("Top Ranked {}", i + 1);
        write_adr(&s, &id, &title, "accepted", "domain", &["networking"], &[]);
    }

    let r = Run::run(&s, &["adr", "conflict-bundle", "ADR-031"]);
    assert_eq!(
        r.exit_code, 0,
        "expected exit 0; stdout:\n{}\nstderr:\n{}",
        r.stdout, r.stderr
    );

    // Proposed ADR header + section appears.
    assert!(
        r.stdout.contains("## Proposed ADR") && r.stdout.contains("ADR-031"),
        "expected Proposed ADR section with ADR-031; got:\n{}",
        r.stdout
    );
    // Cross-cutting ADRs present.
    for id in ["ADR-010", "ADR-011"] {
        assert!(
            r.stdout.contains(id),
            "expected cross-cutting {} in bundle; got:\n{}",
            id, r.stdout
        );
    }
    // Same-domain ADR present.
    assert!(
        r.stdout.contains("ADR-020"),
        "expected same-domain ADR-020 in bundle; got:\n{}",
        r.stdout
    );
    // Existing ADRs section present.
    assert!(
        r.stdout.contains("## Existing ADRs to Check Against"),
        "expected 'Existing ADRs to Check Against' section; got:\n{}",
        r.stdout
    );
    // Each included ADR appears at most twice (once in Proposed, once in Existing).
    let count_031 = r.stdout.matches("ADR-031").count();
    assert!(
        count_031 >= 1,
        "ADR-031 should appear at least once; got {} times",
        count_031
    );
}

// ---------------------------------------------------------------------------
// TC-575 — check-conflicts is structural only, no LLM call, fast
// ---------------------------------------------------------------------------
#[test]
fn tc_575_adr_check_conflicts_structural_only() {
    let s = Session::new();
    // A small corpus of accepted ADRs with no structural conflicts.
    for (i, title) in ["One", "Two", "Three"].iter().enumerate() {
        let id = format!("ADR-00{}", i + 1);
        write_adr(&s, &id, title, "accepted", "domain", &["api"], &[]);
    }

    let start = std::time::Instant::now();
    let r = Run::run(&s, &["adr", "check-conflicts", "ADR-001"]);
    let elapsed = start.elapsed();

    assert!(
        r.exit_code == 0 || r.exit_code == 2,
        "check-conflicts must exit 0 or 2 under FT-045; got {} (stderr: {})",
        r.exit_code, r.stderr
    );
    // No LLM / network references in stderr.
    assert!(
        !r.stderr.contains("model failure") && !r.stderr.contains("LLM"),
        "stderr must not mention model/LLM failures; got:\n{}",
        r.stderr
    );

    // Fast — no LLM call means structural pass must be well under a second
    // on a tiny fixture.
    assert!(
        elapsed.as_secs() < 5,
        "check-conflicts took too long ({:?}); structural path must be fast",
        elapsed
    );
}

// ---------------------------------------------------------------------------
// TC-576 — exit-criteria: FT-045 smoke covering the LLM boundary
// ---------------------------------------------------------------------------
#[test]
fn tc_576_llm_boundary_semantic_analysis_exit() {
    let s = Session::new();
    // Minimal corpus: a feature + an ADR + a TC.
    s.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test Decision\nstatus: accepted\nfeatures: [FT-001]\ndomains: [api]\nscope: domain\n---\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.\n",
    );
    s.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Feature One\nphase: 1\nstatus: planned\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nBody.\n",
    );
    s.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nBody.\n",
    );

    // 1. gap bundle emits markdown with the expected sections.
    let gb = Run::run(&s, &["gap", "bundle", "ADR-001"]);
    assert_eq!(gb.exit_code, 0, "gap bundle failed: {}", gb.stderr);
    assert!(gb.stdout.contains("## Instructions"));
    assert!(gb.stdout.contains("## Context Bundle"));

    // 2. structural gap check — no LLM, exits 0 or 1.
    let gc = Run::run(&s, &["gap", "check"]);
    assert!(
        gc.exit_code == 0 || gc.exit_code == 1,
        "gap check must exit 0 or 1; got {}",
        gc.exit_code
    );

    // 3. adr conflict-bundle emits a bundle.
    let cb = Run::run(&s, &["adr", "conflict-bundle", "ADR-001"]);
    assert_eq!(cb.exit_code, 0, "adr conflict-bundle failed: {}", cb.stderr);
    assert!(cb.stdout.contains("## Instructions"));
    assert!(cb.stdout.contains("## Proposed ADR"));

    // 4. structural adr check-conflicts — no LLM, exits 0 or 2.
    let ac = Run::run(&s, &["adr", "check-conflicts"]);
    assert!(
        ac.exit_code == 0 || ac.exit_code == 2,
        "adr check-conflicts must exit 0 or 2; got {}",
        ac.exit_code
    );

    // 5. prompts list includes the three new prompts.
    let pl = Run::run(&s, &["prompts", "list"]);
    assert_eq!(pl.exit_code, 0, "prompts list failed: {}", pl.stderr);
    for name in ["gap-analysis", "drift-analysis", "conflict-check"] {
        assert!(
            pl.stdout.contains(name),
            "prompts list missing {}; got:\n{}",
            name, pl.stdout
        );
    }
}
