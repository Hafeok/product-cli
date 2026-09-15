use super::*;

pub(crate) fn act(id: &str, entry_points: &[&str]) -> Act {
    let mut built = Act {
        form: ACT_FORM.into(),
        id: id.into(),
        name: "Settle a basket".into(),
        settles: "What the customer owes when the basket closes.".into(),
        realised_at: entry_points.iter().map(|e| (*e).to_string()).collect(),
        ratified_by: "emil@example.com".parse().expect("identity parses"),
        ratified_at: DateTime::parse_from_rfc3339("2026-09-15T09:00:00Z")
            .map(|t| t.with_timezone(&Utc))
            .expect("fixed timestamp"),
        from_candidate: Some("cand/x".into()),
        binds: String::new(),
        signature: None,
    };
    built.binds = built.computed_binds();
    built
}

#[test]
fn an_act_seals_its_own_digest() {
    let ratified = act("act/settle-basket", &["ep/one"]);
    assert_eq!(ratified.binds, ratified.computed_binds());
    assert!(ratified.binds.starts_with("sha256:"));
}

#[test]
fn renaming_an_act_breaks_its_binding() {
    let mut ratified = act("act/settle-basket", &["ep/one"]);
    ratified.name = "Something else".into();
    assert_ne!(ratified.binds, ratified.computed_binds());
}

#[test]
fn rewriting_what_an_act_settles_breaks_its_binding() {
    let mut ratified = act("act/settle-basket", &["ep/one"]);
    ratified.settles = "Something else entirely.".into();
    assert_ne!(ratified.binds, ratified.computed_binds());
}

#[test]
fn entry_point_order_is_formatting_not_meaning() {
    let one = act("act/a", &["ep/one", "ep/two"]);
    let other = act("act/a", &["ep/two", "ep/one"]);
    assert_eq!(one.computed_binds(), other.computed_binds());
}

#[test]
fn realisation_is_membership_not_substring() {
    let ratified = act("act/a", &["ep/one"]);
    assert!(ratified.realises("ep/one"));
    assert!(!ratified.realises("ep/on"));
}

#[test]
fn the_act_and_rejection_forms_are_separate_domains() {
    assert_ne!(ACT_FORM, REJECTION_FORM);
}

#[test]
fn a_rejection_seals_its_reason() {
    let mut refused = Rejection {
        form: REJECTION_FORM.into(),
        candidate: "cand/x".into(),
        reason: "A health probe is not an act.".into(),
        principal: "emil@example.com".parse().expect("identity parses"),
        at: Utc::now(),
        binds: String::new(),
        signature: None,
    };
    refused.binds = refused.computed_binds();
    assert_eq!(refused.binds, refused.computed_binds());

    refused.reason = "Changed my mind.".into();
    assert_ne!(refused.binds, refused.computed_binds());
}
