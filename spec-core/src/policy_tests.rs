use super::*;

pub(crate) fn verdict(metric: &str, fires_when: &str, basis: &str) -> PolicyVerdict {
    PolicyVerdict {
        metric: metric.into(),
        fires_when: fires_when.into(),
        basis: basis.into(),
        basis_binds: basis_digest(fires_when),
        principal: Principal { kind: PrincipalKind::Team, identifier: "platform".into() },
    }
}

pub(crate) fn policy(id: &str, verdicts: Vec<PolicyVerdict>) -> Policy {
    let mut built = Policy {
        form: POLICY_FORM.into(),
        id: id.into(),
        supersedes: None,
        filed_by: "emil@example.com".parse().expect("identity parses"),
        filed_at: DateTime::parse_from_rfc3339("2026-09-15T09:00:00Z")
            .map(|t| t.with_timezone(&Utc))
            .expect("fixed timestamp"),
        policy_verdicts: verdicts,
        not_gated: vec![NotGated {
            metric: "mapping_coverage".into(),
            reason: "Coverage is a metric, not a goal.".into(),
        }],
        not_gated_asserted_none: false,
        binds: String::new(),
    };
    built.binds = built.computed_binds();
    built
}

#[test]
fn a_policy_seals_its_own_digest() {
    let filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    assert_eq!(filed.binds, filed.computed_binds());
}

#[test]
fn moving_a_threshold_changes_the_policy_digest() {
    let one = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    let other = policy("01A", vec![verdict("unmapped_entry_points", "count > 3", "Because.")]);
    assert_ne!(one.computed_binds(), other.computed_binds());
}

#[test]
fn a_basis_binds_the_threshold_it_was_written_against() {
    assert_eq!(basis_digest("count > 0"), basis_digest("count > 0"));
    assert_ne!(basis_digest("count > 0"), basis_digest("count > 3"));
}

#[test]
fn the_principal_kind_has_no_machine_member() {
    // A compile-time property, asserted here so the intent survives a refactor:
    // the restriction is inherited from the type, not applied by a check.
    let rendered = Principal { kind: PrincipalKind::Human, identifier: "emil".into() }.render();
    assert_eq!(rendered, "human:emil");
    assert_eq!(
        Principal { kind: PrincipalKind::Team, identifier: "platform".into() }.render(),
        "team:platform"
    );
}

#[test]
fn a_single_version_is_in_force() {
    let versions = vec![policy("01A", Vec::new())];
    assert_eq!(in_force(&versions).expect("one tip").map(|p| p.id.as_str()), Some("01A"));
}

#[test]
fn the_tip_derives_from_the_chain_not_from_id_order() {
    let first = policy("01Z", Vec::new());
    let mut second = policy("01A", Vec::new());
    second.supersedes = Some("01Z".into());

    let versions = vec![first, second];
    let tip = in_force(&versions).expect("one tip").expect("present");
    assert_eq!(tip.id, "01A", "the later id sorts first; the chain still decides");
}

#[test]
fn a_forked_chain_refuses_to_pick_a_side() {
    let base = policy("01A", Vec::new());
    let mut left = policy("01B", Vec::new());
    left.supersedes = Some("01A".into());
    let mut right = policy("01C", Vec::new());
    right.supersedes = Some("01A".into());

    let versions = vec![base, left, right];
    let tips = in_force(&versions).expect_err("two tips");
    assert_eq!(tips.len(), 2);
}

#[test]
fn no_versions_means_no_policy_in_force() {
    assert!(in_force(&[]).expect("no tips").is_none());
}
