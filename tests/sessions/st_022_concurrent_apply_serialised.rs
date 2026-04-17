//! ST-022 — concurrent applies are serialised by the advisory lock.
//!
//! Validates TC-541 (chaos). When another process holds the
//! `.product.lock` file, a new `product request apply` invocation must
//! fail with E010 rather than proceed and corrupt state. This test
//! simulates the held-lock condition directly (rather than racing two
//! subprocesses) so the outcome is deterministic: the lock holds, the
//! apply is blocked, the graph remains valid.

use super::harness::Session;
use std::process::{Command, Stdio};

/// TC-541 — session ST-022 concurrent-apply-serialised by advisory lock.
#[test]
fn tc_541_session_st_022_concurrent_apply_serialised_by_advisory_lock() {
    let mut s = Session::new();

    // Seed a target feature.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed"
artifacts:
  - type: feature
    title: Target
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    let before = s.docs_digest();

    // Create a lock file manually, simulating another Product process
    // holding the lock. The PID is our own test process so the stale-lock
    // detector won't clear it during the apply attempt.
    let lock_path = s.dir.path().join(".product.lock");
    let holder_pid = std::process::id();
    std::fs::write(
        &lock_path,
        format!(
            "pid={}\nstarted=2026-04-17T00:00:00Z\n",
            holder_pid
        ),
    )
    .expect("write lock");

    // Write a valid change request.
    s.write(
        "a.yaml",
        "type: change\nschema-version: 1\nreason: \"blocked apply\"\nchanges:\n  - target: FT-001\n    mutations:\n      - op: append\n        field: domains\n        value: security\n",
    );

    // Attempt to apply — must fail with E010 because the lock is held.
    let out = Command::new(&s.bin)
        .args(["request", "apply", "a.yaml"])
        .current_dir(s.dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("spawn");
    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(code, 0, "apply must fail while another process holds the lock; got code {}", code);
    assert!(
        stderr.contains("E010") || stderr.contains("locked"),
        "expected E010 (repository locked) — got stderr:\n{}",
        stderr
    );

    // No files changed while the apply was blocked.
    let after = s.docs_digest();
    assert_eq!(before, after, "blocked apply must not mutate docs/");

    // Release the simulated lock and confirm a subsequent apply succeeds
    // — proving the lock (not something else) was the gate.
    std::fs::remove_file(&lock_path).ok();

    // After lock release, the apply succeeds.
    let r = s.apply_file("a.yaml");
    r.assert_applied();

    // And the final graph is valid.
    s.assert_graph_clean();
}
