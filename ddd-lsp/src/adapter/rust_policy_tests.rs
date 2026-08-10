//! Rust policy-table tests — one trigger plus one non-trigger per row.
//!
//! The table is the falsifiable part of the adapter, so every row is
//! exercised from both sides: the change it claims is boundary-forming, and
//! the neighbouring change it claims is not.

use super::*;
use crate::adapter::rust_facts::facts;
use crate::classify::diff_facts;
use crate::protocol::RawSymbol;
use crate::surface::SurfaceEvent;

fn sym(name: &str, kind: u64, container: &str, container_kind: u64, line: u64) -> RawSymbol {
    RawSymbol {
        name: name.to_string(),
        kind,
        container: container.to_string(),
        container_kind,
        start_line: line,
        end_line: line,
        sel_line: line,
        sel_char: 0,
        detail: None,
    }
}

fn flags() -> AdapterFlags {
    AdapterFlags::default()
}

/// Classify a before/after pair the way `classify_edit` does, but with
/// hand-built symbol lists on each side.
fn events(
    before: (&str, Vec<RawSymbol>),
    after: (&str, Vec<RawSymbol>),
) -> Vec<SurfaceEvent> {
    let f = flags();
    diff_facts(
        &policy(&f),
        visibility_rank,
        facts(before.0, &before.1, &f),
        facts(after.0, &after.1, &f),
    )
}

fn rule_for(events: &[SurfaceEvent], name: &str) -> (bool, Option<&'static str>) {
    let e = events
        .iter()
        .find(|e| e.facts.name == name)
        .unwrap_or_else(|| panic!("no event for {name}"));
    (e.surface, e.rule)
}

#[test]
fn adding_a_pub_fn_is_surface_and_adding_a_private_one_is_not() {
    let ev = events(
        ("pub fn a() {}\n", vec![sym("a", 12, "", 0, 0)]),
        (
            "pub fn a() {}\npub fn b() {}\nfn c() {}\n",
            vec![sym("a", 12, "", 0, 0), sym("b", 12, "", 0, 1), sym("c", 12, "", 0, 2)],
        ),
    );
    assert_eq!(rule_for(&ev, "b"), (true, Some("rs-add-exposed")));
    assert_eq!(rule_for(&ev, "c"), (false, Some("rs-private-nonsurface")));
}

#[test]
fn a_pub_crate_change_is_not_surface_by_default_but_flips_with_the_config() {
    let before = ("pub(crate) fn a() {}\n", vec![sym("a", 12, "", 0, 0)]);
    let after =
        ("pub(crate) fn a(x: u32) {}\n", vec![sym("a", 12, "", 0, 0)]);
    assert_eq!(rule_for(&events(before, after), "a"), (false, Some("rs-crate-visible-nonsurface")));

    let mut flipped = flags();
    flipped.internal_is_surface = true;
    let ev = diff_facts(
        &policy(&flipped),
        visibility_rank,
        facts("pub(crate) fn a() {}\n", &[sym("a", 12, "", 0, 0)], &flipped),
        facts("pub(crate) fn a(x: u32) {}\n", &[sym("a", 12, "", 0, 0)], &flipped),
    );
    assert_eq!(rule_for(&ev, "a"), (true, Some("rs-crate-visible-surface")));
}

#[test]
fn a_signature_change_on_a_pub_fn_is_surface() {
    let ev = events(
        ("pub fn a(x: u32) {}\n", vec![sym("a", 12, "", 0, 0)]),
        ("pub fn a(x: u64) {}\n", vec![sym("a", 12, "", 0, 0)]),
    );
    assert_eq!(rule_for(&ev, "a"), (true, Some("rs-signature-exposed")));
}

#[test]
fn a_generic_bound_change_is_a_signature_change() {
    let ev = events(
        ("pub fn a<T: Clone>(x: T) {}\n", vec![sym("a", 12, "", 0, 0)]),
        ("pub fn a<T: Clone + Send>(x: T) {}\n", vec![sym("a", 12, "", 0, 0)]),
    );
    assert_eq!(rule_for(&ev, "a"), (true, Some("rs-signature-exposed")));
}

#[test]
fn a_body_change_alone_produces_no_event() {
    let ev = events(
        ("pub fn a() -> u32 { 1 }\n", vec![sym("a", 12, "", 0, 0)]),
        ("pub fn a() -> u32 { 2 }\n", vec![sym("a", 12, "", 0, 0)]),
    );
    assert!(ev.is_empty(), "{ev:?}");
}

#[test]
fn adding_a_trait_member_forces_implementors() {
    let before = (
        "pub trait T {\n    fn a(&self);\n}\n",
        vec![sym("T", 11, "", 0, 0), sym("a", 6, "T", 11, 1)],
    );
    let after = (
        "pub trait T {\n    fn a(&self);\n    fn b(&self);\n}\n",
        vec![sym("T", 11, "", 0, 0), sym("a", 6, "T", 11, 1), sym("b", 6, "T", 11, 2)],
    );
    assert_eq!(rule_for(&events(before, after), "b"), (true, Some("rs-trait-member")));
}

#[test]
fn a_member_on_a_private_trait_is_not_surface() {
    let before =
        ("trait T {\n    fn a(&self);\n}\n", vec![sym("T", 11, "", 0, 0), sym("a", 6, "T", 11, 1)]);
    let after = (
        "trait T {\n    fn a(&self);\n    fn b(&self);\n}\n",
        vec![sym("T", 11, "", 0, 0), sym("a", 6, "T", 11, 1), sym("b", 6, "T", 11, 2)],
    );
    assert_eq!(rule_for(&events(before, after), "b"), (false, Some("rs-private-nonsurface")));
}

#[test]
fn adding_a_trait_impl_is_boundary_forming_without_any_pub_keyword() {
    let before = ("pub struct P;\n", vec![sym("P", 23, "", 0, 0)]);
    let after = (
        "pub struct P;\npub trait G {}\nimpl G for P {}\n",
        vec![
            sym("P", 23, "", 0, 0),
            sym("G", 11, "", 0, 1),
            sym("impl G for P", 19, "", 0, 2),
        ],
    );
    let ev = events(before, after);
    assert_eq!(rule_for(&ev, "impl G for P"), (true, Some("rs-trait-impl")));
}

#[test]
fn adding_an_impl_on_a_private_type_is_not_surface() {
    let before = ("struct P;\n", vec![sym("P", 23, "", 0, 0)]);
    let after = (
        "struct P;\npub trait G {}\nimpl G for P {}\n",
        vec![sym("P", 23, "", 0, 0), sym("G", 11, "", 0, 1), sym("impl G for P", 19, "", 0, 2)],
    );
    assert_eq!(
        rule_for(&events(before, after), "impl G for P"),
        (false, Some("rs-private-nonsurface"))
    );
}

#[test]
fn an_unresolved_impl_is_demanded_regardless_of_visibility() {
    let before = ("// nothing\n", vec![]);
    let after = (
        "impl Display for Elsewhere {}\n",
        vec![sym("impl Display for Elsewhere", 19, "", 0, 0)],
    );
    assert_eq!(
        rule_for(&events(before, after), "impl Display for Elsewhere"),
        (true, Some("rs-trait-impl-unresolved"))
    );
}

#[test]
fn adding_a_variant_to_a_pub_enum_is_surface() {
    let before = (
        "pub enum E {\n    A,\n}\n",
        vec![sym("E", 10, "", 0, 0), sym("A", 22, "E", 10, 1)],
    );
    let after = (
        "pub enum E {\n    A,\n    B,\n}\n",
        vec![sym("E", 10, "", 0, 0), sym("A", 22, "E", 10, 1), sym("B", 22, "E", 10, 2)],
    );
    // The gap DDD-adapter-03 reports for C# is adapter-local; this table
    // does not inherit it.
    assert_eq!(rule_for(&events(before, after), "B"), (true, Some("rs-add-exposed")));
}

#[test]
fn a_derive_change_on_a_pub_type_is_surface() {
    let before = (
        "#[derive(Clone)]\npub struct P;\n",
        vec![sym("P", 23, "", 0, 1)],
    );
    let after = (
        "#[derive(Clone, Debug)]\npub struct P;\n",
        vec![sym("P", 23, "", 0, 1)],
    );
    assert_eq!(rule_for(&events(before, after), "P"), (true, Some("rs-derive-change")));
}

#[test]
fn a_derive_change_on_a_private_type_is_not_surface() {
    let before = ("#[derive(Clone)]\nstruct P;\n", vec![sym("P", 23, "", 0, 1)]);
    let after = ("#[derive(Clone, Debug)]\nstruct P;\n", vec![sym("P", 23, "", 0, 1)]);
    assert_eq!(rule_for(&events(before, after), "P"), (false, Some("rs-private-nonsurface")));
}

#[test]
fn removing_pub_surface_is_a_contract_change() {
    let before = (
        "pub fn a() {}\npub fn b() {}\n",
        vec![sym("a", 12, "", 0, 0), sym("b", 12, "", 0, 1)],
    );
    let after = ("pub fn a() {}\n", vec![sym("a", 12, "", 0, 0)]);
    assert_eq!(rule_for(&events(before, after), "b"), (true, Some("rs-remove-exposed")));
}

#[test]
fn demoting_a_pub_fn_is_judged_at_the_more_exposed_side() {
    let ev = events(
        ("pub fn a() {}\n", vec![sym("a", 12, "", 0, 0)]),
        ("fn a() {}\n", vec![sym("a", 12, "", 0, 0)]),
    );
    assert_eq!(rule_for(&ev, "a"), (true, Some("rs-visibility-boundary")));
}

#[test]
fn a_trait_definition_change_names_the_trait_row() {
    let ev = events(
        ("pub trait G {}\n", vec![sym("G", 11, "", 0, 0)]),
        ("pub trait G: Clone {}\n", vec![sym("G", 11, "", 0, 0)]),
    );
    assert_eq!(rule_for(&ev, "G"), (true, Some("rs-trait-definition")));
}

#[test]
fn every_row_carries_a_claim() {
    for row in policy(&flags()) {
        assert!(!row.claim.trim().is_empty(), "{} has no claim", row.id);
        assert!(row.id.starts_with("rs-"), "{}", row.id);
    }
}
