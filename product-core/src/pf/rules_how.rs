//! SPARQL conformance rules for the How layer — contracts, archetypes, layout.
//!
//! Each rule mirrors a `sh:SPARQLConstraint` in `schema/shapes/how.shacl.ttl`:
//! the crown trace rule plus the cross-reference checks (`licenses`, `realizes`,
//! `conformsTo`, `realized_by`, layout `enforces`) that every edge must resolve.
//! These replace the native field-walks in `how_validate`/`archetype`; the
//! presence/cardinality checks stay native there.

use super::sparql_rules::SparqlRule;

/// The crown rule (§5/§4.1): every principle a work unit applies must be
/// enforced by some verification. Mirrors `pf:TraceTruthShape` verbatim.
pub const TRACE_TRUTH: SparqlRule = SparqlRule {
    id: "trace-truth",
    focus_var: "wu",
    path: "trace",
    severity: "violation",
    select: r#"
      SELECT ?wu ?principle WHERE {
        ?wu a <https://productframework.org/ns#WorkUnit> .
        ?wu <https://productframework.org/ns#applies> ?principle .
        ?principle a <https://productframework.org/ns#Principle> .
        FILTER NOT EXISTS {
          ?v a <https://productframework.org/ns#Verification> .
          ?v <https://productframework.org/ns#enforces> ?principle .
        }
      }
    "#,
    message: |r| format!(
        "§5/§4.1 The trace must be true: a work unit applies '{}', which no verification enforces — back it or retract it.",
        r.get("principle").map(String::as_str).unwrap_or("?")),
};

/// §4.1 Earn-their-place: a principle must be applied by a work unit or enforced
/// by a verification, else it is documentation, not architecture.
pub const EARN_THEIR_PLACE: SparqlRule = SparqlRule {
    id: "earn-their-place",
    focus_var: "principle",
    path: "earn-their-place",
    severity: "violation",
    select: r#"
      SELECT ?principle WHERE {
        ?principle a <https://productframework.org/ns#Principle> .
        FILTER NOT EXISTS { ?wu a <https://productframework.org/ns#WorkUnit> ; <https://productframework.org/ns#applies> ?principle . }
        FILTER NOT EXISTS { ?v a <https://productframework.org/ns#Verification> ; <https://productframework.org/ns#enforces> ?principle . }
      }
    "#,
    message: |_| "§4.1 Earn-their-place: a principle must be applied by a work unit or enforced by a verification, else it is documentation, not architecture.".to_string(),
};

/// §4.1 A pattern's `realizes` must reference a Principle that exists.
pub const REALIZES_RESOLVES: SparqlRule = SparqlRule {
    id: "realizes-resolves",
    focus_var: "pattern",
    path: "realizes",
    severity: "violation",
    select: r#"
      SELECT ?pattern ?principle WHERE {
        ?pattern a <https://productframework.org/ns#Pattern> .
        ?pattern <https://productframework.org/ns#realizes> ?principle .
        FILTER NOT EXISTS { ?principle a <https://productframework.org/ns#Principle> . }
      }
    "#,
    message: |r| format!(
        "§4.1 A pattern's realizes must reference a Principle that exists — '{}' does not.",
        r.get("principle").map(String::as_str).unwrap_or("?")),
};

/// §4.2 An infrastructure contract must satisfy (conformsTo) the application
/// contract — its target must be a declared ApplicationContract.
pub const CONFORMS_TO: SparqlRule = SparqlRule {
    id: "conforms-to",
    focus_var: "infra",
    path: "conformsTo",
    severity: "violation",
    select: r#"
      SELECT ?infra ?target WHERE {
        ?infra a <https://productframework.org/ns#InfrastructureContract> .
        ?infra <https://productframework.org/ns#conformsTo> ?target .
        FILTER NOT EXISTS { ?target a <https://productframework.org/ns#ApplicationContract> . }
      }
    "#,
    message: |_| "§4.2 An infrastructure contract must satisfy (conformsTo) the application contract.".to_string(),
};

/// §4.1 A top decision's `licenses` should each reference a defined Principle.
pub const LICENSES_RESOLVES: SparqlRule = SparqlRule {
    id: "licenses-resolves",
    focus_var: "decision",
    path: "licenses",
    severity: "warning",
    select: r#"
      SELECT ?decision ?target WHERE {
        ?decision a <https://productframework.org/ns#TopDecision> .
        ?decision <https://productframework.org/ns#licenses> ?target .
        FILTER NOT EXISTS { ?target a <https://productframework.org/ns#Principle> . }
      }
    "#,
    message: |r| format!(
        "§4.1 A top decision's licenses should each reference a defined Principle — '{}' is undefined.",
        r.get("target").map(String::as_str).unwrap_or("?")),
};

/// §4.1 A principle's `realized_by` should reference a defined Pattern.
pub const REALIZED_BY_RESOLVES: SparqlRule = SparqlRule {
    id: "realized-by-resolves",
    focus_var: "principle",
    path: "realized_by",
    severity: "warning",
    select: r#"
      SELECT ?principle ?target WHERE {
        ?principle a <https://productframework.org/ns#Principle> .
        ?principle <https://productframework.org/ns#realizedBy> ?target .
        FILTER NOT EXISTS { ?target a <https://productframework.org/ns#Pattern> . }
      }
    "#,
    message: |r| format!(
        "§4.1 A principle's realized_by should reference a defined Pattern — '{}' is undefined.",
        r.get("target").map(String::as_str).unwrap_or("?")),
};

/// §4.3 Guard 1 / §5 honesty: a layout rule's `enforces` must resolve to a
/// principle or decision the How defines, else the rationale it cites is a
/// dangling reference.
pub const ENFORCES_RESOLVES: SparqlRule = SparqlRule {
    id: "enforces-resolves",
    focus_var: "rule",
    path: "enforces",
    severity: "warning",
    select: r#"
      SELECT ?rule ?target WHERE {
        ?rule a <https://productframework.org/ns#LayoutRule> .
        ?rule <https://productframework.org/ns#enforces> ?target .
        FILTER NOT EXISTS { ?target a <https://productframework.org/ns#Principle> . }
        FILTER NOT EXISTS { ?target a <https://productframework.org/ns#TopDecision> . }
      }
    "#,
    message: |r| format!(
        "rule '{}' enforces '{}', which is not a principle or decision in the How",
        r.get("rule").map(String::as_str).unwrap_or("?"),
        r.get("target").map(String::as_str).unwrap_or("?")),
};

/// The cross-reference + trace rules over a How contract's projection.
pub fn how_rules() -> &'static [SparqlRule] {
    &[
        TRACE_TRUTH,
        EARN_THEIR_PLACE,
        REALIZES_RESOLVES,
        CONFORMS_TO,
        LICENSES_RESOLVES,
        REALIZED_BY_RESOLVES,
    ]
}

/// The rules over an assembled archetype's combined (How + layout) projection.
pub fn archetype_rules() -> &'static [SparqlRule] {
    &[ENFORCES_RESOLVES]
}

#[cfg(test)]
mod tests {
    use super::super::sparql_rules::run_rules;
    use super::*;

    const PREFIXES: &str = "@prefix pf: <https://productframework.org/ns#> .\n@prefix d: <https://productframework.org/archetype/x#> .\n";

    #[test]
    fn trace_truth_passes_when_enforced() {
        let ttl = format!("{PREFIXES}d:P1 a pf:Principle .\nd:V1 a pf:Verification ; pf:enforces d:P1 .\nd:WU1 a pf:WorkUnit ; pf:applies d:P1 .\n");
        assert!(run_rules(&ttl, &[TRACE_TRUTH]).is_empty());
    }

    #[test]
    fn trace_truth_fires_on_dangling_apply() {
        let ttl = format!("{PREFIXES}d:P1 a pf:Principle .\nd:WU1 a pf:WorkUnit ; pf:applies d:P1 .\n");
        let vs = run_rules(&ttl, &[TRACE_TRUTH]);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].path, "trace");
        assert!(vs[0].message.contains("trace must be true"));
    }

    #[test]
    fn enforces_resolves_fires_on_ghost_and_accepts_a_decision() {
        let ghost = format!("{PREFIXES}d:r a pf:LayoutRule ; pf:enforces d:ghost-principle .\n");
        let vs = run_rules(&ghost, &[ENFORCES_RESOLVES]);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].severity, "warning");
        assert!(vs[0].message.contains("ghost-principle"));

        let ok = format!("{PREFIXES}d:D1 a pf:TopDecision .\nd:r a pf:LayoutRule ; pf:enforces d:D1 .\n");
        assert!(run_rules(&ok, &[ENFORCES_RESOLVES]).is_empty());
    }

    #[test]
    fn earn_their_place_fires_on_orphan_principle() {
        let ttl = format!("{PREFIXES}d:P1 a pf:Principle .\n");
        let vs = run_rules(&ttl, &[EARN_THEIR_PLACE]);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].focus, "P1");
    }
}
