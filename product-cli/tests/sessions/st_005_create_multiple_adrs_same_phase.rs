//! ST-005 — multiple ADRs created in one request receive IDs in order.
//!
//! Validates TC-537.

use super::harness::Session;

/// TC-537 — session ST-005 create-multiple-adrs-same-phase assigns IDs in order.
#[test]
fn tc_537_session_st_005_create_multiple_adrs_same_phase_assigns_ids_in_order() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-005 — multiple ADRs"
artifacts:
  - type: adr
    ref: adr-one
    title: First Decision
    domains: [api]
    scope: domain
  - type: adr
    ref: adr-two
    title: Second Decision
    domains: [api]
    scope: domain
  - type: adr
    ref: adr-three
    title: Third Decision
    domains: [api]
    scope: domain
"#,
    );
    r.assert_applied();
    assert_eq!(r.created.len(), 3);

    let one = r.id_for("adr-one");
    let two = r.id_for("adr-two");
    let three = r.id_for("adr-three");

    // IDs are sequential and ordered — one < two < three numerically.
    let num = |s: &str| {
        s.trim_start_matches("ADR-")
            .parse::<u32>()
            .expect("numeric ADR id")
    };
    assert!(
        num(&one) < num(&two) && num(&two) < num(&three),
        "expected ordered IDs; got {} {} {}",
        one,
        two,
        three
    );
    assert_eq!(num(&two), num(&one) + 1);
    assert_eq!(num(&three), num(&two) + 1);

    s.assert_graph_clean();
}
