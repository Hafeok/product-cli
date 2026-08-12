//! Rust facts tests — visibility grading, signature slicing, impl shapes.
//!
//! Symbols are hand-built in the shapes rust-analyzer 1.95.0 actually emits
//! (kind numbers, `impl Trait for Type` names, `container_kind`). The
//! real-host test in `tests/rust_host.rs` asserts the live server still
//! emits those shapes, so these fixtures cannot drift silently.

use super::*;
use crate::adapter::rust_facts::facts;
use crate::protocol::RawSymbol;

fn sym(name: &str, kind: u64, container: &str, container_kind: u64, line: u64) -> RawSymbol {
    sym_span(name, kind, container, container_kind, line, line)
}

/// A symbol whose range spans several lines, the way rust-analyzer reports
/// any declaration with a body: `end` is the item's last line, which caps
/// how far the signature slice may read (the corpus-caught bleed fix).
fn sym_span(
    name: &str,
    kind: u64,
    container: &str,
    container_kind: u64,
    line: u64,
    end: u64,
) -> RawSymbol {
    RawSymbol {
        name: name.to_string(),
        kind,
        container: container.to_string(),
        container_kind,
        start_line: line,
        end_line: end,
        sel_line: line,
        sel_char: 0,
        detail: None,
    }
}

fn flags() -> AdapterFlags {
    AdapterFlags::default()
}

fn facts_of(text: &str, symbols: &[RawSymbol]) -> Vec<crate::surface::SymbolFacts> {
    facts(text, symbols, &flags())
}

fn find<'a>(
    list: &'a [crate::surface::SymbolFacts],
    name: &str,
) -> &'a crate::surface::SymbolFacts {
    list.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("{name} not in facts"))
}

// ---------------------------------------------------------------- visibility

#[test]
fn grades_every_visibility_the_language_writes() {
    let text = "\
pub fn open() {}
pub(crate) fn crate_wide() {}
pub(super) fn parental() {}
pub(in crate::a) fn restricted() {}
pub(self) fn selfish() {}
fn hidden() {}
";
    let symbols: Vec<RawSymbol> = ["open", "crate_wide", "parental", "restricted", "selfish", "hidden"]
        .iter()
        .enumerate()
        .map(|(i, n)| sym(n, 12, "", 0, i as u64))
        .collect();
    let f = facts_of(text, &symbols);
    assert_eq!(find(&f, "open").visibility, "public");
    assert_eq!(find(&f, "crate_wide").visibility, "pub(crate)");
    assert_eq!(find(&f, "parental").visibility, "pub(super)");
    assert_eq!(find(&f, "restricted").visibility, "pub(restricted)");
    // `pub(self)` is private written the long way.
    assert_eq!(find(&f, "selfish").visibility, "private");
    assert_eq!(find(&f, "hidden").visibility, "private");
}

#[test]
fn a_pub_item_in_a_private_module_is_capped() {
    let text = "\
mod inner {
    pub fn reachable_only_here() {}
}
";
    let symbols =
        vec![sym("inner", 2, "", 0, 0), sym("reachable_only_here", 12, "inner", 2, 1)];
    let f = facts_of(text, &symbols);
    assert_eq!(find(&f, "reachable_only_here").visibility, "private");
}

#[test]
fn trait_members_and_enum_variants_inherit_their_container() {
    let text = "\
pub trait Greeter {
    fn greet(&self) -> String;
}
pub enum Colour {
    Red,
}
";
    let symbols = vec![
        sym("Greeter", 11, "", 0, 0),
        sym("greet", 6, "Greeter", 11, 1),
        sym("Colour", 10, "", 0, 3),
        sym("Red", 22, "Colour", 10, 4),
    ];
    let f = facts_of(text, &symbols);
    assert_eq!(find(&f, "greet").visibility, "public");
    assert_eq!(find(&f, "greet").kind, "trait-member");
    assert_eq!(find(&f, "Red").visibility, "public");
    assert_eq!(find(&f, "Red").kind, "enum-member");
}

// --------------------------------------------------------------- signatures

#[test]
fn a_signature_keeps_bounds_and_where_clauses_but_drops_the_body() {
    let text = "\
pub fn convert<T: Into<String>>(t: T) -> Result<String, Error>
where
    T: Clone,
{
    unimplemented!()
}
";
    let f = facts_of(text, &[sym_span("convert", 12, "", 0, 0, 5)]);
    assert_eq!(
        find(&f, "convert").signature,
        "fn convert<T: Into<String>>(t: T) -> Result<String, Error> where T: Clone,"
    );
}

/// The corpus-caught false-demand source: an enum member has no `{`/`;`/`=`
/// terminator, so an unbounded slice would absorb the lines below the enum —
/// then any edit down there reported a phantom signature change on the
/// member. The symbol's own range end caps the slice instead.
#[test]
fn a_terminator_less_declaration_never_reads_past_its_own_range() {
    let before = "pub enum Colour {\n    Red,\n}\n\npub fn a() -> u32 { 1 }\n";
    let after = "pub enum Colour {\n    Red,\n}\n\npub fn b(x: u32) -> u32 { x }\n";
    let symbols = [
        sym_span("Colour", 10, "", 0, 0, 2),
        sym("Red", 22, "Colour", 10, 1),
    ];
    let sig_before = find(&facts_of(before, &symbols), "Red").signature.clone();
    let sig_after = find(&facts_of(after, &symbols), "Red").signature.clone();
    assert_eq!(sig_before, "Red,");
    assert_eq!(sig_before, sig_after, "an unchanged member must keep its signature");
}

#[test]
fn the_arrow_in_a_return_type_is_not_a_closing_angle_bracket() {
    let text = "pub fn ids() -> Vec<u32> { vec![] }\n";
    let f = facts_of(text, &[sym("ids", 12, "", 0, 0)]);
    assert_eq!(find(&f, "ids").signature, "fn ids() -> Vec<u32>");
}

#[test]
fn a_default_type_parameter_is_not_an_initialiser() {
    let text = "pub struct Holder<T = u32> { inner: T }\n";
    let f = facts_of(text, &[sym("Holder", 23, "", 0, 0)]);
    assert_eq!(find(&f, "Holder").signature, "struct Holder<T = u32>");
}

// -------------------------------------------------------------------- impls

#[test]
fn a_trait_impl_is_read_off_the_symbol_name() {
    let text = "\
pub struct Person;
impl Greeter for Person {
    fn greet(&self) -> String { String::new() }
}
";
    let symbols = vec![
        sym("Person", 23, "", 0, 0),
        sym("impl Greeter for Person", 19, "", 0, 1),
        sym("greet", 6, "impl Greeter for Person", 19, 2),
    ];
    let f = facts_of(text, &symbols);
    let imp = find(&f, "impl Greeter for Person");
    assert_eq!(imp.kind, "trait-impl");
    // No `pub` on an impl block: judged at the type it is written for.
    assert_eq!(imp.visibility, "public");
    assert_eq!(imp.extra.get("trait").map(String::as_str), Some("Greeter"));
    assert_eq!(imp.extra.get("self_type").map(String::as_str), Some("Person"));
}

#[test]
fn an_impl_on_a_private_type_is_judged_private() {
    let text = "\
struct Hidden;
impl Greeter for Hidden {}
";
    let symbols =
        vec![sym("Hidden", 23, "", 0, 0), sym("impl Greeter for Hidden", 19, "", 0, 1)];
    let f = facts_of(text, &symbols);
    assert_eq!(find(&f, "impl Greeter for Hidden").visibility, "private");
}

#[test]
fn an_impl_with_neither_side_local_is_flagged_unresolved() {
    let text = "impl std::fmt::Display for OtherFile {}\n";
    let symbols = vec![sym("impl Display for OtherFile", 19, "", 0, 0)];
    let f = facts_of(text, &symbols);
    let imp = find(&f, "impl Display for OtherFile");
    assert_eq!(imp.kind, "trait-impl-unresolved");
    assert!(imp.extra.contains_key("coherence"), "{:?}", imp.extra);
}

#[test]
fn an_inherent_impl_is_not_trait_participation() {
    let text = "\
pub struct Person;
impl<T> Person<T> {}
";
    let symbols = vec![sym("Person", 23, "", 0, 0), sym("impl<T> Person<T>", 19, "", 0, 1)];
    let f = facts_of(text, &symbols);
    assert_eq!(find(&f, "impl<T> Person<T>").kind, "inherent-impl");
}

// ------------------------------------------------------------------ posture

#[test]
fn a_re_export_raises_the_capping_caveat() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("lib.rs");
    std::fs::write(&file, "mod inner { pub struct T; }\npub use inner::T;\n").expect("write");
    let warning = posture_warning(&file, &flags()).expect("warning");
    assert!(warning.contains("re-export"), "{warning}");

    std::fs::write(&file, "pub struct T;\n").expect("write");
    assert!(posture_warning(&file, &flags()).is_none());
}

// ------------------------------------------------------------- ready signal

#[test]
fn readiness_is_the_payload_not_the_arrival() {
    let ReadySignal::NotificationWhere(method, predicate) = ADAPTER.ready else {
        panic!("rust readiness should be payload-discriminated");
    };
    assert_eq!(method, "experimental/serverStatus");
    // rust-analyzer sends this first, while indexing.
    assert!(!predicate(&json!({"health": "ok", "quiescent": false})));
    assert!(predicate(&json!({"health": "ok", "quiescent": true})));
    // A host that says nothing about quiescence is not ready.
    assert!(!predicate(&json!({})));
}

#[test]
fn the_server_status_capability_is_announced() {
    let caps = (ADAPTER.extra_capabilities)();
    assert_eq!(caps["experimental"]["serverStatusNotification"], json!(true));
}
