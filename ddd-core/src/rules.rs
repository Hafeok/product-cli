//! SPARQL ontology rules over the `ddd:` projection (PRD §6, rules 2–5).
//!
//! Each rule is a `sh:sparql`-style SELECT returning one row per violation,
//! run by `pf::sparql_rules::run_rules`. Rule 1 (predicates carry no status)
//! is enforced at the schema layer in `store` — the projection can never see
//! a field the schema already refused, so no query could express it.
//!
//! Row bindings arrive percent-encoded (IRI local names); messages decode
//! them via `turtle::decode_ref`, and `validate` decodes each row's focus.

use std::collections::BTreeMap;

use product_core::pf::sparql_rules::SparqlRule;

use crate::turtle::decode_ref;

fn bound(row: &BTreeMap<String, String>, var: &str) -> String {
    row.get(var).map(|v| decode_ref(v)).unwrap_or_else(|| "?".to_string())
}

/// §6 rule 2 — `depends_on` edges are acyclic (predicates plus claims).
pub const DEPENDS_ON_ACYCLIC: SparqlRule = SparqlRule {
    id: "ddd-depends-on-acyclic",
    focus_var: "x",
    path: "depends_on",
    severity: "violation",
    select: r#"
      SELECT ?x WHERE {
        ?x <https://decisiondriven.design/ns#dependsOn>+ ?x .
      }
    "#,
    message: |_| "§6 rule 2: depends_on edges must be acyclic — this entry sits on a dependency cycle".to_string(),
};

/// §6 rule 2 — `refines` edges are acyclic.
pub const REFINES_ACYCLIC: SparqlRule = SparqlRule {
    id: "ddd-refines-acyclic",
    focus_var: "x",
    path: "refines",
    severity: "violation",
    select: r#"
      SELECT ?x WHERE {
        ?x <https://decisiondriven.design/ns#refines>+ ?x .
      }
    "#,
    message: |_| "§6 rule 2: refines edges must be acyclic — this entry sits on a refinement cycle".to_string(),
};

/// §6 rule 2 — `refines` targets must exist.
pub const REFINES_TARGET_EXISTS: SparqlRule = SparqlRule {
    id: "ddd-refines-target-exists",
    focus_var: "c",
    path: "refines",
    severity: "violation",
    select: r#"
      SELECT ?c ?t WHERE {
        ?c <https://decisiondriven.design/ns#refines> ?t .
        FILTER NOT EXISTS { ?t a <https://decisiondriven.design/ns#Claim> }
      }
    "#,
    message: |row| format!("§6 rule 2: refines target '{}' does not exist in claims/", bound(row, "t")),
};

/// §6 rule 3 — every decision has at least one `basedOn` edge.
pub const DECISION_HAS_BASIS: SparqlRule = SparqlRule {
    id: "ddd-decision-has-basis",
    focus_var: "d",
    path: "based_on",
    severity: "violation",
    select: r#"
      SELECT ?d WHERE {
        ?d a <https://decisiondriven.design/ns#Decision> .
        FILTER NOT EXISTS { ?d <https://decisiondriven.design/ns#basedOn> ?c }
      }
    "#,
    message: |_| "§6 rule 3: every decision needs at least one basedOn edge to an existing claim — none declared".to_string(),
};

/// §6 rule 3 — every `basedOn` edge lands on an existing claim.
pub const BASIS_EXISTS: SparqlRule = SparqlRule {
    id: "ddd-basis-exists",
    focus_var: "d",
    path: "based_on",
    severity: "violation",
    select: r#"
      SELECT ?d ?c WHERE {
        ?d <https://decisiondriven.design/ns#basedOn> ?c .
        FILTER NOT EXISTS { ?c a <https://decisiondriven.design/ns#Claim> }
      }
    "#,
    message: |row| format!("§6 rule 3: basedOn claim '{}' does not exist in claims/", bound(row, "c")),
};

/// §6 rule 3 — every volitional entry names its principal.
pub const PRINCIPAL_NAMED: SparqlRule = SparqlRule {
    id: "ddd-principal-named",
    focus_var: "d",
    path: "principal",
    severity: "violation",
    select: r#"
      SELECT ?d WHERE {
        { ?d a <https://decisiondriven.design/ns#Decision> }
        UNION
        { ?d a <https://decisiondriven.design/ns#RiskAcceptance> }
        FILTER NOT EXISTS { ?d <https://decisiondriven.design/ns#principal> ?p }
      }
    "#,
    message: |_| "§6 rule 3: a volitional entry must name its principal".to_string(),
};

/// §6 rule 4 — every manifest entry maps to a decision (or is UNGOVERNED).
pub const MANIFEST_GOVERNED: SparqlRule = SparqlRule {
    id: "ddd-manifest-governed",
    focus_var: "m",
    path: "decision",
    severity: "violation",
    select: r#"
      SELECT ?m WHERE {
        ?m a <https://decisiondriven.design/ns#ManifestEntry> .
        FILTER NOT EXISTS { ?m <https://decisiondriven.design/ns#governedBy> ?d }
        FILTER NOT EXISTS { ?m <https://decisiondriven.design/ns#ungoverned> ?u }
      }
    "#,
    message: |_| "§6 rule 4: manifest entry maps to no decision — set `decision:` or mark it UNGOVERNED".to_string(),
};

/// §6 rule 4 — a governing decision reference must resolve.
pub const MANIFEST_DECISION_EXISTS: SparqlRule = SparqlRule {
    id: "ddd-manifest-decision-exists",
    focus_var: "m",
    path: "decision",
    severity: "violation",
    select: r#"
      SELECT ?m ?d WHERE {
        ?m <https://decisiondriven.design/ns#governedBy> ?d .
        FILTER NOT EXISTS { ?d a <https://decisiondriven.design/ns#Decision> }
      }
    "#,
    message: |row| format!("§6 rule 4: governing decision '{}' does not exist in decisions/", bound(row, "d")),
};

/// §6 rule 4 — UNGOVERNED entries surface as warnings pending a decision.
pub const UNGOVERNED_PENDING: SparqlRule = SparqlRule {
    id: "ddd-ungoverned-pending",
    focus_var: "m",
    path: "decision",
    severity: "warning",
    select: r#"
      SELECT ?m WHERE {
        ?m <https://decisiondriven.design/ns#ungoverned> ?u .
      }
    "#,
    message: |_| "§6 rule 4: entry is UNGOVERNED — pending a decision entry".to_string(),
};

/// §6 rule 4 — every suppression cites a risk-acceptance record.
pub const SUPPRESSION_CITES: SparqlRule = SparqlRule {
    id: "ddd-suppression-cites",
    focus_var: "m",
    path: "suppression",
    severity: "violation",
    select: r#"
      SELECT ?m WHERE {
        ?m <https://decisiondriven.design/ns#suppressed> ?s .
        FILTER NOT EXISTS { ?m <https://decisiondriven.design/ns#cites> ?r }
      }
    "#,
    message: |_| "§6 rule 4: a suppression must cite a risk-acceptance record".to_string(),
};

/// §6 rule 4 — a cited record must be a risk acceptance that exists.
pub const CITATION_IS_RISK_ACCEPTANCE: SparqlRule = SparqlRule {
    id: "ddd-citation-is-risk-acceptance",
    focus_var: "m",
    path: "suppression",
    severity: "violation",
    select: r#"
      SELECT ?m ?r WHERE {
        ?m <https://decisiondriven.design/ns#cites> ?r .
        FILTER NOT EXISTS { ?r a <https://decisiondriven.design/ns#RiskAcceptance> }
      }
    "#,
    message: |row| format!(
        "§6 rule 4: cited risk acceptance '{}' does not exist in decisions/ (kind: risk-acceptance)",
        bound(row, "r")
    ),
};

/// §6 rule 5 — a pattern instance references its pattern predicate.
pub const PATTERN_HAS_PREDICATE: SparqlRule = SparqlRule {
    id: "ddd-pattern-has-predicate",
    focus_var: "p",
    path: "pattern_predicate",
    severity: "violation",
    select: r#"
      SELECT ?p WHERE {
        ?p a <https://decisiondriven.design/ns#PatternInstance> .
        FILTER NOT EXISTS { ?p <https://decisiondriven.design/ns#instantiates> ?q }
      }
    "#,
    message: |_| "§6 rule 5: a pattern instance must reference a pattern predicate in the catalog".to_string(),
};

/// §6 rule 5 — the referenced pattern predicate exists in the catalog.
pub const PATTERN_PREDICATE_EXISTS: SparqlRule = SparqlRule {
    id: "ddd-pattern-predicate-exists",
    focus_var: "p",
    path: "pattern_predicate",
    severity: "violation",
    select: r#"
      SELECT ?p ?q WHERE {
        ?p <https://decisiondriven.design/ns#instantiates> ?q .
        FILTER NOT EXISTS { ?q a <https://decisiondriven.design/ns#Predicate> }
      }
    "#,
    message: |row| format!("§6 rule 5: pattern predicate '{}' does not exist in predicates/", bound(row, "q")),
};

/// Every ontology rule, in reporting order.
pub static ONTOLOGY_RULES: &[SparqlRule] = &[
    DEPENDS_ON_ACYCLIC,
    REFINES_ACYCLIC,
    REFINES_TARGET_EXISTS,
    DECISION_HAS_BASIS,
    BASIS_EXISTS,
    PRINCIPAL_NAMED,
    MANIFEST_GOVERNED,
    MANIFEST_DECISION_EXISTS,
    UNGOVERNED_PENDING,
    SUPPRESSION_CITES,
    CITATION_IS_RISK_ACCEPTANCE,
    PATTERN_HAS_PREDICATE,
    PATTERN_PREDICATE_EXISTS,
];
