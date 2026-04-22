//! ST-061 — `product context <FT> --depth 2` expands beyond direct links to
//! include ADRs that govern the dependencies and related features. A shared
//! ADR that FT-A's dep points to is reachable at depth 2 from FT-A even though
//! FT-A doesn't link it directly.
//!
//! Validates TC-679.

use super::harness::Session;

/// TC-679 — session ST-061 context-depth-2-includes-shared-adrs.
#[test]
fn tc_679_session_st_061_context_depth_2_includes_shared_adrs() {
    let mut s = Session::new();

    // Graph shape:
    //   FT-main ──links──> ADR-direct
    //   DEP-shared ──linked-by──> ADR-direct, ADR-deep
    //
    // FT-main links only ADR-direct. At depth 1 from FT-main the bundle
    // reaches ADR-direct + DEP-shared. At depth 2 it reaches ADR-deep via
    // the dep's second ADR link.
    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-061 — shared ADR via dep at depth 2"
artifacts:
  - type: adr
    ref: adr-direct
    title: Direct Decision
    domains: [api]
  - type: adr
    ref: adr-deep
    title: Deep Shared Decision
    domains: [api]
  - type: feature
    ref: ft-main
    title: Depth Feature
    phase: 1
    domains: [api]
    adrs: [ref:adr-direct]
  - type: dep
    ref: dep-shared
    title: Shared Library
    dep-type: library
    version: "1.0"
    adrs: [ref:adr-direct, ref:adr-deep]
"#,
    );
    r.assert_applied();
    let ft = r.id_for("ft-main");
    let adr_direct = r.id_for("adr-direct");
    let adr_deep = r.id_for("adr-deep");

    // Depth 1: direct ADR is present, deep ADR is NOT.
    let depth1 = s.run(&["context", &ft, "--depth", "1"]);
    depth1.assert_exit(0);
    assert!(
        depth1.stdout.contains(&adr_direct),
        "depth=1 bundle must include the directly-linked ADR {adr_direct}; got:\n{}",
        depth1.stdout
    );

    // Depth 2: deep ADR surfaces through the shared dep.
    let depth2 = s.run(&["context", &ft, "--depth", "2"]);
    depth2.assert_exit(0);
    assert!(
        depth2.stdout.contains(&adr_direct),
        "depth=2 bundle must still include the directly-linked ADR {adr_direct}; got:\n{}",
        depth2.stdout
    );
    assert!(
        depth2.stdout.contains(&adr_deep),
        "depth=2 bundle must include the shared ADR {adr_deep} reachable via the dep; got:\n{}",
        depth2.stdout
    );
}
