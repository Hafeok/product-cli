use super::*;
use crate::metrics::Metric;
use crate::policy::tests::{policy, verdict};
use crate::policy::{NotGated, Policy};

fn metrics() -> BTreeMap<String, Metric> {
    BTreeMap::from([
        ("unmapped_entry_points".into(), Metric::out_of(2, 5)),
        ("mapping_coverage".into(), Metric::out_of(3, 5)),
        ("records_open".into(), Metric::count(0)),
        ("nothing_to_cover".into(), Metric::out_of(0, 0)),
    ])
}

fn classes(findings: &[Finding]) -> Vec<Class> {
    findings.iter().map(|f| f.class).collect()
}

#[test]
fn a_well_formed_policy_has_no_structural_findings() {
    let filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    assert!(judge_policy(&filed, &metrics()).is_empty());
}

#[test]
fn s008_a_verdict_without_a_basis_is_malformed() {
    let filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "")]);
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S008));
}

#[test]
fn s008_a_verdict_naming_no_metric_is_malformed() {
    let filed = policy("01A", vec![verdict("invented_metric", "count > 0", "Because.")]);
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S008));
}

#[test]
fn s008_an_unevaluable_condition_is_malformed() {
    let filed = policy("01A", vec![verdict("records_open", "roughly a lot", "Because.")]);
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S008));
}

#[test]
fn s009_moving_a_threshold_without_the_argument_fails() {
    let mut filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    // The number moves; the basis, and the digest it carries, do not.
    filed.policy_verdicts[0].fires_when = "count > 50".into();
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S009));
}

#[test]
fn s009_revisiting_the_argument_clears_it() {
    let filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 50", "Rewritten.")]);
    assert!(!classes(&judge_policy(&filed, &metrics())).contains(&Class::S009));
}

#[test]
fn s010_one_argument_repeated_fails() {
    let filed = policy("01A", vec![
        verdict("unmapped_entry_points", "count > 0", "The same words."),
        verdict("records_open", "count > 0", "The same words."),
    ]);
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S010));
}

#[test]
fn two_verdicts_with_genuinely_different_arguments_pass() {
    let filed = policy("01A", vec![
        verdict("unmapped_entry_points", "count > 0", "One argument."),
        verdict("records_open", "count > 0", "A different argument entirely."),
    ]);
    assert!(!classes(&judge_policy(&filed, &metrics())).contains(&Class::S010));
}

#[test]
fn s011_a_policy_with_no_uncovered_set_fails() {
    let mut filed = policy("01A", vec![verdict("records_open", "count > 0", "Because.")]);
    filed.not_gated = Vec::new();
    assert!(classes(&judge_policy(&filed, &metrics())).contains(&Class::S011));
}

#[test]
fn an_explicit_assertion_of_none_is_admitted() {
    let mut filed = policy("01A", vec![verdict("records_open", "count > 0", "Because.")]);
    filed.not_gated = Vec::new();
    filed.not_gated_asserted_none = true;
    assert!(!classes(&judge_policy(&filed, &metrics())).contains(&Class::S011));
}

#[test]
fn a_verdict_fires_when_its_condition_holds() {
    let filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    let fired = run(&filed, &metrics());
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].observed, "2 / 5 (40%)");
}

#[test]
fn a_verdict_stays_quiet_when_its_condition_does_not_hold() {
    let filed = policy("01A", vec![verdict("records_open", "count > 0", "Because.")]);
    assert!(run(&filed, &metrics()).is_empty());
}

#[test]
fn a_malformed_verdict_never_fires() {
    let mut filed = policy("01A", vec![verdict("unmapped_entry_points", "count > 0", "Because.")]);
    filed.policy_verdicts[0].fires_when = "count > 0".into();
    filed.policy_verdicts[0].basis_binds = "sha256:wrong".into();
    assert!(
        run(&filed, &metrics()).is_empty(),
        "it already produced a structural finding; firing too would report one defect twice"
    );
}

#[test]
fn percentage_conditions_read_the_proportion() {
    let filed = policy("01A", vec![verdict("mapping_coverage", "percent < 80", "Because.")]);
    assert_eq!(run(&filed, &metrics()).len(), 1);
}

#[test]
fn a_percentage_over_nothing_never_fires() {
    let filed = policy("01A", vec![verdict("nothing_to_cover", "percent < 80", "Because.")]);
    assert!(
        run(&filed, &metrics()).is_empty(),
        "firing on an undefined figure would be a verdict about the absence of data"
    );
}

#[test]
fn the_condition_grammar_is_deliberately_tiny() {
    assert!(Condition::parse("count > 0").is_some());
    assert!(Condition::parse("percent <= 80%").is_some());
    assert!(Condition::parse("count").is_none());
    assert!(Condition::parse("count > 0 and something").is_none());
    assert!(Condition::parse("coverage > 0").is_none());
}

#[test]
fn the_default_policy_is_structural_only() {
    let empty = Policy {
        policy_verdicts: Vec::new(),
        not_gated: vec![NotGated {
            metric: "mapping_coverage".into(),
            reason: "Coverage is a metric, not a goal.".into(),
        }],
        ..policy("01A", Vec::new())
    };
    assert!(run(&empty, &metrics()).is_empty());
    assert!(judge_policy(&empty, &metrics()).is_empty());
}
