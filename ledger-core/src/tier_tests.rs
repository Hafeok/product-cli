//! Tolerance cases: the up-only rule, the effective tier, floor raising.

use super::*;

#[test]
fn tiers_order_by_consequence() {
    assert!(Tier::T0 < Tier::T1 && Tier::T1 < Tier::T2);
}

#[test]
fn an_override_above_the_floor_is_constructible() {
    let t = Tolerance::new(Tier::T1, Some(Tier::T2)).expect("up-only");
    assert_eq!(t.effective(), Tier::T2);
    assert_eq!(t.floor_at_creation(), Tier::T1);
    assert_eq!(t.override_tier(), Some(Tier::T2));
}

#[test]
fn a_below_floor_override_is_unconstructible() {
    let err = Tolerance::new(Tier::T2, Some(Tier::T0)).expect_err("below floor");
    assert_eq!(err, BelowFloor { over: Tier::T0, floor: Tier::T2 });
    assert!(err.to_string().contains("is below the set floor T2"), "{err}");
}

#[test]
fn an_override_equal_to_the_floor_is_rejected_as_a_no_op() {
    let err = Tolerance::new(Tier::T1, Some(Tier::T1)).expect_err("not above");
    assert!(err.to_string().contains("equals the set floor T1"), "{err}");
}

#[test]
fn without_an_override_the_effective_tier_is_the_pinned_floor() {
    let t = Tolerance::new(Tier::T1, None).expect("no override");
    assert_eq!(t.effective(), Tier::T1);
}

#[test]
fn raising_the_floor_strands_a_member_below_it() {
    let member = Tolerance::new(Tier::T0, None).expect("t0 member");
    assert!(!member.is_stranded_below(Tier::T0));
    assert!(member.is_stranded_below(Tier::T1), "T0 member under a raised T1 floor");
}

#[test]
fn a_member_already_overridden_upward_survives_a_floor_raise() {
    // The pin does not move, so the hash does not move, so the acceptance
    // stands — and the member is not stranded because its effective tier
    // already meets the new floor.
    let member = Tolerance::new(Tier::T0, Some(Tier::T2)).expect("overridden up");
    assert!(!member.is_stranded_below(Tier::T2));
    assert_eq!(member.floor_at_creation(), Tier::T0, "the pin is history, not the live floor");
}
