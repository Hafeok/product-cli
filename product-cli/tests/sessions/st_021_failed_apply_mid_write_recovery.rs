//! ST-021 — mid-apply failure leaves the working tree recoverable.
//!
//! Validates TC-540 (chaos). Because we cannot inject fault handlers into
//! a spawned child process without modifying product itself, this session
//! exercises the observable half of the property: a request that is
//! well-formed up to validation but fails in a later stage (e.g. an
//! invalid ref target that passes parse but fails resolution) leaves the
//! existing files untouched and no sidecar `.product-tmp.*` files behind.

use super::harness::Session;

/// TC-540 — session ST-021 failed-apply-mid-write-recovery.
#[test]
fn tc_540_session_st_021_failed_apply_mid_write_recovery() {
    let mut s = Session::new();

    // Seed initial state.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed"
artifacts:
  - type: feature
    title: Keep Me
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    let before = s.docs_digest();

    // A create request whose cross-reference cannot be resolved. This
    // fails after parse during validation/resolution — exercising the
    // rollback path.
    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-021 — unresolved ref"
artifacts:
  - type: feature
    ref: ft-orphan
    title: Orphan
    phase: 1
    domains: [api]
    adrs: [ref:does-not-exist]
"#,
    );
    r.assert_failed();

    // Zero-files-changed invariant.
    let after = s.docs_digest();
    assert_eq!(before, after, "rollback must restore pre-apply state");

    // No sidecar files left behind under docs/.
    let mut leftover = Vec::new();
    let docs = s.dir.path().join("docs");
    let mut stack = vec![docs];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name.contains(".product-tmp.") {
                        leftover.push(p.display().to_string());
                    }
                }
            }
        }
    }
    assert!(
        leftover.is_empty(),
        "no .product-tmp.* sidecar files should remain; found: {:?}",
        leftover
    );
}
