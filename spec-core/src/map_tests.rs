use super::*;
use crate::act::tests::act;
use crate::inventory::tests::sample;
use crate::Act;

const SETTLE: &str = "Shop.Api.BasketController.Settle#HttpPost";

fn store(acts: Vec<Act>) -> SpecStore {
    SpecStore { acts, inventory: Some(sample()), ..SpecStore::default() }
}

#[test]
fn one_act_at_one_entry_point_says_nothing() {
    let joined = join(&store(vec![act("act/settle-basket", &[SETTLE])]));
    assert!(joined.is_empty(), "{joined:?}");
}

#[test]
fn several_entry_points_under_one_act_is_a_merge() {
    let joined = join(&store(vec![act("act/settle-basket", &[SETTLE, "ep/other"])]));
    assert!(joined.iter().any(|f| matches!(f, MapFinding::Merge { entry_points, .. }
        if entry_points.len() == 2)));
}

#[test]
fn several_acts_at_one_entry_point_is_a_split() {
    let joined = join(&store(vec![act("act/a", &[SETTLE]), act("act/b", &[SETTLE])]));
    let split = joined.iter().find(|f| matches!(f, MapFinding::Split { .. }));
    assert_eq!(
        split,
        Some(&MapFinding::Split {
            entry_point: SETTLE.into(),
            acts: vec!["act/a".into(), "act/b".into()],
        })
    );
}

#[test]
fn an_uncovered_entry_point_is_unmapped() {
    let joined = join(&store(Vec::new()));
    assert!(joined.iter().any(|f| matches!(f, MapFinding::UnmappedEntryPoint { entry_point, .. }
        if entry_point == SETTLE)));
}

#[test]
fn an_act_realised_nowhere_the_scan_sees_is_unrealised() {
    let joined = join(&store(vec![act("act/ghost", &["ep/vanished"])]));
    assert!(joined.contains(&MapFinding::UnmappedAct { act: "act/ghost".into() }));
}

#[test]
fn an_act_claiming_no_entry_point_is_unrealised_without_an_inventory() {
    let store = SpecStore { acts: vec![act("act/ghost", &[])], ..SpecStore::default() };
    assert_eq!(join(&store), vec![MapFinding::UnmappedAct { act: "act/ghost".into() }]);
}

#[test]
fn findings_render_with_the_restructuring_the_act_justifies() {
    let merge = MapFinding::Merge {
        act: "act/settle-basket".into(),
        entry_points: vec!["ep/one".into(), "ep/two".into()],
    };
    assert!(merge.line().contains("act/settle-basket ← 2 entry points"), "{}", merge.line());
}

#[test]
fn the_join_is_ordered_for_a_reviewer_not_for_the_data() {
    let joined = join(&store(vec![
        act("act/merged", &[SETTLE, "ep/other"]),
        act("act/ghost", &["ep/vanished"]),
    ]));
    let merge_at = joined.iter().position(|f| matches!(f, MapFinding::Merge { .. }));
    let unrealised_at = joined.iter().position(|f| matches!(f, MapFinding::UnmappedAct { .. }));
    assert!(merge_at < unrealised_at);
}
