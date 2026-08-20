//! The record of how one assertion was read — §7.1's recoverability argument.
//!
//! G0's forcing case: 31 assertions were rejected because their properties came
//! from members the server reported as *fields*, and **nothing in a ratified
//! assertion recorded that**. The count had to be taken from the extractor's
//! own JSON, so a reviewer re-examining that class of error could not find its
//! members. That is ground that cannot be audited.
//!
//! The remedy is not a judgement rule. It is to record the facts the reader
//! actually saw — the rule, the operations it stood on, the source kinds — and
//! let the reviewer's *query* do the classification. Group A (generated code)
//! becomes a predicate over [`Derivation::source_paths`]; Group B becomes
//! `member_kind = field`. Neither requires the extractor to judge anything,
//! neither filters, neither widens the LSP subset.
//!
//! These facts are **evidence, not ratified claim**: they live in the run
//! sidecar (`super::sidecar`), keyed by the assertion's own id, leaving the
//! canonical graph pure ratified assertion. They are also *recomputable* — the
//! extractor is deterministic and the corpus ref is pinned — which is what
//! makes the already-ratified cohort auditable after the fact.

use serde::Serialize;

use super::facts::{DeclKind, MemberKind};

/// How one assertion came to be proposed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Derivation {
    /// The named rule inside the row. A row may carry several, and "which
    /// row" is too coarse to batch on.
    pub rule: &'static str,
    /// The declaration-level operations the assertion's facts came from, as
    /// probed. An assertion standing on an unanswered operation cannot exist,
    /// and this says which ones it does stand on.
    pub stands_on: &'static [&'static str],
    /// The kind of member the assertion derives from, where it derives from
    /// one. **The fact whose absence made Group B unfindable.**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_kind: Option<MemberKind>,
    /// The kind of declaration containing what the assertion is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_kind: Option<DeclKind>,
    /// Repo-relative paths of the declarations the assertion derives from.
    /// `reg:evidence` already carries paths, mixed in with type names and rule
    /// names; this is the typed form a query can stand on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_paths: Vec<String>,
}

impl Derivation {
    /// A derivation standing on `stands_on`, produced by `rule`.
    pub fn by(rule: &'static str, stands_on: &'static [&'static str]) -> Self {
        Derivation {
            rule,
            stands_on,
            member_kind: None,
            container_kind: None,
            source_paths: Vec::new(),
        }
    }

    /// The declaration the assertion is about, or whose member it is about.
    pub fn in_container(mut self, kind: DeclKind, path: &str) -> Self {
        self.container_kind = Some(kind);
        self.source_paths.push(path.to_string());
        self
    }

    /// The member the assertion derives from, as the server reported it —
    /// reported, never judged.
    pub fn of_member(mut self, kind: MemberKind) -> Self {
        self.member_kind = Some(kind);
        self
    }

    /// A further source position, for a rule that joins two declarations.
    pub fn also_at(mut self, path: &str) -> Self {
        if !self.source_paths.iter().any(|p| p == path) {
            self.source_paths.push(path.to_string());
        }
        self
    }

    /// The batching signature: everything about the derivation that is
    /// constant across a batch. Paths vary per assertion, so they are the
    /// batch's *contents* rather than part of its key.
    pub(crate) fn signature(&self) -> (&'static str, Option<MemberKind>, Option<DeclKind>) {
        (self.rule, self.member_kind, self.container_kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPS: &[&str] = &["documentSymbol"];

    #[test]
    fn a_bare_derivation_names_its_rule_with_the_operations_it_stood_on() {
        let d = Derivation::by("declared-type", OPS);
        assert_eq!(d.rule, "declared-type");
        assert_eq!(d.stands_on, ["documentSymbol"]);
        assert_eq!(d.member_kind, None);
        assert!(d.source_paths.is_empty());
    }

    /// The Group B fact, recorded rather than absorbed: a member the server
    /// called a field is distinguishable afterwards from one it called a
    /// property, which is exactly what a ratified assertion could not say.
    #[test]
    fn a_field_derived_assertion_is_distinguishable_from_a_property_derived_one() {
        let field = Derivation::by("declared-member", OPS)
            .in_container(DeclKind::Class, "src/Converters/Foo.cs")
            .of_member(MemberKind::Field);
        let property = Derivation::by("declared-member", OPS)
            .in_container(DeclKind::Class, "src/Entities/Bar.cs")
            .of_member(MemberKind::Property);
        assert_ne!(field.signature(), property.signature());
        assert_eq!(field.member_kind, Some(MemberKind::Field));
    }

    /// Two assertions from the same rule over the same kinds share a
    /// signature even when they sit in different files — the file is the
    /// batch's content, not its key.
    #[test]
    fn the_signature_is_stable_across_source_positions() {
        let one = Derivation::by("declared-type", OPS).in_container(DeclKind::Record, "a.cs");
        let two = Derivation::by("declared-type", OPS).in_container(DeclKind::Record, "b.cs");
        assert_eq!(one.signature(), two.signature());
        assert_ne!(one.source_paths, two.source_paths);
    }

    #[test]
    fn a_repeated_source_position_is_recorded_once() {
        let d = Derivation::by("usage-construct", OPS)
            .in_container(DeclKind::Class, "a.cs")
            .also_at("b.cs")
            .also_at("a.cs");
        assert_eq!(d.source_paths, ["a.cs", "b.cs"]);
    }
}
