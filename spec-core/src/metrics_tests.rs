use super::*;
use crate::act::tests::act;
use crate::inventory::tests::sample;

const SETTLE: &str = "Shop.Api.BasketController.Settle#HttpPost";

#[test]
fn nothing_to_cover_reads_none_not_a_hundred_percent() {
    assert_eq!(Metric::out_of(0, 0).percent(), None);
    assert_eq!(Metric::out_of(0, 0).render(), "0 / 0");
}

#[test]
fn a_proportion_renders_with_its_denominator() {
    assert_eq!(Metric::out_of(1, 4).render(), "1 / 4 (25%)");
    assert_eq!(Metric::count(3).render(), "3");
}

#[test]
fn an_empty_store_computes_every_metric_at_zero() {
    let metrics = compute(&SpecStore::default());
    assert!(metrics.values().all(|m| m.count == 0));
    assert!(metrics.contains_key("mapping_coverage"));
}

#[test]
fn coverage_tracks_ratified_acts() {
    let store = SpecStore { inventory: Some(sample()), ..SpecStore::default() };
    assert_eq!(compute(&store)["mapping_coverage"].percent(), Some(0.0));

    let covered = SpecStore {
        acts: vec![act("act/settle", &[SETTLE])],
        inventory: Some(sample()),
        ..SpecStore::default()
    };
    assert_eq!(compute(&covered)["mapping_coverage"].percent(), Some(100.0));
    assert_eq!(compute(&covered)["unmapped_entry_points"].count, 0);
}

#[test]
fn an_act_realised_nowhere_the_scan_sees_counts_as_unrealised() {
    let store = SpecStore {
        acts: vec![act("act/ghost", &["ep/vanished"])],
        inventory: Some(sample()),
        ..SpecStore::default()
    };
    assert_eq!(compute(&store)["acts_unrealised"].count, 1);
}
