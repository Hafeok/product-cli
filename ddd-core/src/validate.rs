//! Validation orchestration — native shape checks, then the SPARQL rules.
//!
//! Splits along the same line as `pf::validate`: presence/cardinality checks
//! (format versions, duplicate ids, the claim-format status ladder) stay
//! native here, while the cross-reference ontology rules (PRD §6 rules 2–5)
//! run as SPARQL over the Turtle projection. Schema faults collected at load
//! time are carried through, so one pass reports the whole store.

use std::collections::BTreeMap;

use product_core::pf::sparql_rules::run_rules;
use product_core::pf::validate::Violation;

use crate::claim::{Claim, ClaimStatus};
use crate::rules::ONTOLOGY_RULES;
use crate::store::DddStore;
use crate::turtle;

/// The entry format versions this tool validates against; each entry is
/// checked against the version it declares (PRD §6).
pub const SUPPORTED_FORMATS: &[u32] = &[1, 2, 3, 4, 5, 6, 7];

/// The decision format that carries reopen edges (`revisit_if`), adopting
/// the ledger's spec-v1.5 shape into this store. See `crate::revisit`.
pub const REVISIT_FORMAT: u32 = 7;

/// Run every check over the store; empty means conformant.
pub fn validate_store(store: &DddStore) -> Vec<Violation> {
    let mut out = store.schema_violations.clone();
    out.extend(format_checks(store));
    out.extend(format_gate_checks(store));
    out.extend(duplicate_id_checks(store));
    out.extend(claim_shape_checks(&store.claims));
    out.extend(tolerance_checks(store));
    let mut sparql = run_rules(&turtle::to_turtle(store), ONTOLOGY_RULES);
    for v in &mut sparql {
        v.focus = turtle::decode_ref(&v.focus);
    }
    out.extend(sparql);
    out
}

/// How many of these violations block (warnings do not).
pub fn blocking(violations: &[Violation]) -> usize {
    violations.iter().filter(|v| v.severity != "warning").count()
}

fn fault(focus: &str, path: &str, message: String) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message, severity: "violation".to_string() }
}

/// Validation is always against the declared format; v1 plus v2 are known.
fn format_checks(store: &DddStore) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut check = |id: &str, format: u32| {
        if !SUPPORTED_FORMATS.contains(&format) {
            out.push(fault(
                id,
                "format",
                format!("declares format {format}; this tool validates formats 1-7"),
            ));
        }
    };
    for p in &store.predicates {
        check(&p.id, p.format);
    }
    for c in &store.claims {
        check(&c.id, c.format);
    }
    for d in &store.decisions {
        check(&d.id, d.format);
    }
    for m in &store.manifests {
        check(&m.name, m.file.format);
    }
    for p in &store.patterns {
        check(&p.id, p.format);
    }
    for s in &store.seams {
        check(&s.id, s.format);
    }
    for e in &store.seam_events {
        check(&e.id, e.format);
    }
    if let Some(c) = &store.config {
        check("config.yaml", c.format);
    }
    out
}

/// Format-2 features must be declared: an entry using them under format 1
/// is a violation, so v1 entries never change meaning silently (PRD §6).
fn format_gate_checks(store: &DddStore) -> Vec<Violation> {
    let mut out = Vec::new();
    out.extend(store.claims.iter().flat_map(claim_format_gates));
    out.extend(store.decisions.iter().flat_map(decision_format_gates));
    out.extend(store.seams.iter().flat_map(seam_format_gates));
    if let Some(c) = &store.config {
        out.extend(config_gate_checks(c));
    }
    for p in &store.predicates {
        if p.format < 3 && !p.obligations.is_empty() {
            out.push(fault(
                &p.id,
                "obligations",
                "pattern obligations are a format-3 field — declare `format: 3`".to_string(),
            ));
        }
    }
    out
}

/// Seam gates: signed bindings are seam format 2 (M8), and a stored
/// binding hash that does not recompute is a violation — a binding whose
/// hash has drifted signs nothing.
fn seam_format_gates(s: &crate::seam::SeamDeclaration) -> Vec<Violation> {
    let mut out = Vec::new();
    if s.format < 2 && !s.bindings.is_empty() {
        out.push(fault(
            &s.id,
            "bindings",
            "signed bindings are a format-2 field — declare `format: 2`".to_string(),
        ));
    }
    for b in &s.bindings {
        if b.hash != b.computed_hash() {
            out.push(fault(
                &s.id,
                "bindings",
                format!(
                    "binding for `{}` in {} stores hash {} but its content hashes to {} — the binding signs nothing",
                    b.symbol,
                    b.file,
                    b.hash,
                    b.computed_hash()
                ),
            ));
        }
    }
    out
}

/// Claim gates: `revalidate_by` is format 2, `version_index` format 4, and
/// an index that names no version anchors nothing.
fn claim_format_gates(c: &Claim) -> Vec<Violation> {
    let mut out = Vec::new();
    if c.format < 2 && c.revalidate_by.is_some() {
        out.push(fault(
            &c.id,
            "revalidate_by",
            "revalidate_by is a format-2 field — declare `format: 2`".to_string(),
        ));
    }
    match &c.version_index {
        Some(_) if c.format < 4 => out.push(fault(
            &c.id,
            "version_index",
            "version_index is a format-4 field — declare `format: 4`".to_string(),
        )),
        Some(v) if v.is_empty() => out.push(fault(
            &c.id,
            "version_index",
            "a version_index must anchor at least one of language/sdk/target/tool".to_string(),
        )),
        _ => {}
    }
    out
}

/// Decision gates: pins are format 2, and a format-2 decision pins every
/// claim edge or none (rule 6) — a half-pinned decision hides its basis
/// loss. Typed bases are format 5; a format-5 decision types every basis,
/// a claim basis always pins, and a non-claim basis states what it is.
fn decision_format_gates(d: &crate::decision::Decision) -> Vec<Violation> {
    use crate::decision::BasedOn;
    let mut out = Vec::new();
    let mut push = |path: &str, message: String| out.push(fault(&d.id, path, message));
    let claim_edges = d.based_on.iter().filter(|b| b.claim_id().is_some()).count();
    let pinned = d.based_on.iter().filter(|b| b.pin().is_some()).count();
    let typed = d.based_on.iter().filter(|b| b.is_typed()).count();
    if d.format < 2 && pinned > 0 {
        push("based_on", "pinned basedOn edges are format 2 — declare `format: 2`".to_string());
    }
    if d.format >= 2 && pinned < claim_edges {
        push(
            "based_on",
            "a format-2 decision pins every basedOn claim edge with the claim's status plus changed date at decision time (rule 6)".to_string(),
        );
    }
    if d.format < 5 && typed > 0 {
        push("based_on", "typed bases are format 5 — declare `format: 5`".to_string());
    }
    if d.format >= 5 && typed < d.based_on.len() {
        push(
            "based_on",
            "a format-5 decision types every basis (claim | constraint | mandate | preference | experiment | risk-acceptance)".to_string(),
        );
    }
    let content_pins = d.based_on.iter().filter(|b| b.pin().is_some_and(|p| p.content.is_some())).count();
    if d.format < 6 && content_pins > 0 {
        push(
            "based_on",
            "content-hash pins are format 6 — declare `format: 6`".to_string(),
        );
    }
    if d.format >= 6 && content_pins < pinned {
        push(
            "based_on",
            "a format-6 decision pins every claim edge with the claim's content hash (M8, invariant 2)".to_string(),
        );
    }
    for basis in &d.based_on {
        if let BasedOn::Typed(t) = basis {
            out.extend(typed_basis_gates(&d.id, t));
        }
    }
    out.extend(crate::revisit::gates(d));
    out
}

/// A typed non-claim basis states what it rests on: a claim type must pin
/// (so it never parses here), risk-acceptance refs its record, the rest
/// carry a statement.
fn typed_basis_gates(id: &str, t: &crate::decision::TypedBasis) -> Vec<Violation> {
    use crate::decision::BasisType;
    let blank = |o: &Option<String>| o.as_deref().map(str::trim).unwrap_or("").is_empty();
    let mut out = Vec::new();
    match t.basis_type {
        BasisType::Claim => out.push(fault(
            id,
            "based_on",
            "a `type: claim` basis pins claim/status/changed like any claim edge — a claim basis with no pin hides its basis loss".to_string(),
        )),
        BasisType::RiskAcceptance => {
            if blank(&t.reference) {
                out.push(fault(
                    id,
                    "based_on",
                    "a `type: risk-acceptance` basis must `ref:` the risk-acceptance record it rests on".to_string(),
                ));
            }
        }
        _ => {
            if blank(&t.statement) {
                out.push(fault(
                    id,
                    "based_on",
                    format!(
                        "a `type: {}` basis must state what it rests on (`statement:`)",
                        t.basis_type.as_str()
                    ),
                ));
            }
        }
    }
    out
}

/// Config-file gates: sections must be declared at their format, modes
/// must come from the closed vocabulary.
fn config_gate_checks(c: &crate::config::DddConfig) -> Vec<Violation> {
    let mut out = Vec::new();
    if c.format < 2 && (c.diff.is_some() || c.detect.is_some()) {
        out.push(fault(
            "config.yaml",
            "format",
            "diff/detect sections are format 2 — declare `format: 2`".to_string(),
        ));
    }
    if c.format < 3 && (c.intercept_by_class.is_some() || c.adapter.is_some()) {
        out.push(fault(
            "config.yaml",
            "format",
            "intercept_by_class/adapter sections are format 3 — declare `format: 3`".to_string(),
        ));
    }
    let detect_v4 = c
        .detect
        .as_ref()
        .map(|d| !d.stylelint.is_empty() || !d.htmlvalidate.is_empty() || !d.tokens.is_empty())
        .unwrap_or(false);
    if c.format < 4 && (c.pair.is_some() || detect_v4) {
        out.push(fault(
            "config.yaml",
            "format",
            "the pair section plus detect.stylelint/htmlvalidate/tokens are format 4 — declare `format: 4`".to_string(),
        ));
    }
    out.extend(intercept_mode_checks(c));
    out
}

/// Interception modes come from a closed vocabulary (PRD §8).
fn intercept_mode_checks(c: &crate::config::DddConfig) -> Vec<Violation> {
    const MODES: &[&str] = &["enforce", "warn", "off"];
    let mut out = Vec::new();
    if !MODES.contains(&c.intercept.as_str()) {
        out.push(fault(
            "config.yaml",
            "intercept",
            format!("`{}` is not an interception mode (enforce | warn | off)", c.intercept),
        ));
    }
    for (class, mode) in c.intercept_by_class.iter().flatten() {
        if !MODES.contains(&mode.as_str()) {
            out.push(fault(
                "config.yaml",
                "intercept_by_class",
                format!("`{class}: {mode}` is not an interception mode (enforce | warn | off)"),
            ));
        }
    }
    out
}

/// Ids are stable identities; a duplicate makes `why` ambiguous.
fn duplicate_id_checks(store: &DddStore) -> Vec<Violation> {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let ids = store
        .predicates
        .iter()
        .map(|p| p.id.as_str())
        .chain(store.claims.iter().map(|c| c.id.as_str()))
        .chain(store.decisions.iter().map(|d| d.id.as_str()))
        .chain(store.patterns.iter().map(|p| p.id.as_str()))
        .chain(store.seams.iter().map(|s| s.id.as_str()));
    for id in ids {
        *seen.entry(id).or_default() += 1;
    }
    seen.iter()
        .filter(|(_, n)| **n > 1)
        .map(|(id, n)| fault(id, "id", format!("id is filed {n} times — ids are stable, never shared")))
        .collect()
}

/// The claim-format status ladder: a live claim keeps its falsifier; any
/// status resting on exercise keeps its evidence (retired keeps what killed it).
fn claim_shape_checks(claims: &[Claim]) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in claims {
        let blank = |o: &Option<String>| o.as_deref().map(str::trim).unwrap_or("").is_empty();
        if c.status != ClaimStatus::Retired && blank(&c.falsifier) {
            out.push(fault(
                &c.id,
                "falsifier",
                format!("a {} claim must state the observation that would kill it", c.status.as_str()),
            ));
        }
        if c.status != ClaimStatus::Projected && blank(&c.evidence) {
            out.push(fault(
                &c.id,
                "evidence",
                format!("a {} claim must state what its status rests on", c.status.as_str()),
            ));
        }
    }
    out
}

/// `predicate-format.md`: without a tolerance the entry is invalid.
fn tolerance_checks(store: &DddStore) -> Vec<Violation> {
    store
        .predicates
        .iter()
        .filter(|p| p.tolerance.trim().is_empty())
        .map(|p| fault(&p.id, "tolerance", "a predicate must declare its tolerance bound".to_string()))
        .collect()
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
