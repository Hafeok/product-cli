//! SPARQL conformance rules for a Decider — the §3.3 anti-drift checks.
//!
//! The authored signature must not drift from the event model: it decides for a
//! real aggregate, handles no foreign commands, covers every command targeting
//! its aggregate, and emits only events its handled commands sanction. Each is a
//! SELECT over the combined What + Decider projection (`decider_to_turtle`),
//! returning one row per violation.

use std::collections::BTreeSet;

use super::decider::Decider;
use super::decider_logic::Scalar;
use super::sparql_rules::SparqlRule;
use super::validate::Violation;

/// §3.3 A Decider decides for an aggregate entity — the target must be a real
/// Entity in the What graph.
pub const DECIDES_FOR_ENTITY: SparqlRule = SparqlRule {
    id: "decides-for-entity",
    focus_var: "decider",
    path: "decides_for",
    severity: "violation",
    select: r#"
      SELECT ?decider ?agg WHERE {
        ?decider a <https://productframework.org/ns#Decider> .
        ?decider <https://productframework.org/ns#decidesFor> ?agg .
        FILTER NOT EXISTS { ?agg a <https://productframework.org/ns#Entity> . }
      }
    "#,
    message: |r| format!(
        "§3.3 A Decider must decide for a real aggregate entity — '{}' is not an entity.",
        r.get("agg").map(String::as_str).unwrap_or("?")),
};

/// §3.3 No foreign commands — a handled command must target the aggregate.
pub const NO_FOREIGN_COMMANDS: SparqlRule = SparqlRule {
    id: "no-foreign-commands",
    focus_var: "decider",
    path: "handles",
    severity: "violation",
    select: r#"
      SELECT ?decider ?cmd WHERE {
        ?decider a <https://productframework.org/ns#Decider> .
        ?decider <https://productframework.org/ns#decidesFor> ?agg .
        ?decider <https://productframework.org/ns#handles> ?cmd .
        FILTER NOT EXISTS { ?cmd <https://productframework.org/ns#targets> ?agg . }
      }
    "#,
    message: |r| format!(
        "§3.3 No foreign commands: a Decider handles '{}', which does not target its aggregate.",
        r.get("cmd").map(String::as_str).unwrap_or("?")),
};

/// §3.3 Command coverage — every command targeting the aggregate is handled.
pub const COMMAND_COVERAGE: SparqlRule = SparqlRule {
    id: "command-coverage",
    focus_var: "decider",
    path: "handles",
    severity: "violation",
    select: r#"
      SELECT ?decider ?cmd WHERE {
        ?decider a <https://productframework.org/ns#Decider> .
        ?decider <https://productframework.org/ns#decidesFor> ?agg .
        ?cmd a <https://productframework.org/ns#Command> .
        ?cmd <https://productframework.org/ns#targets> ?agg .
        FILTER NOT EXISTS { ?decider <https://productframework.org/ns#handles> ?cmd . }
      }
    "#,
    message: |r| format!(
        "§3.3 Command coverage: command '{}' targets the aggregate but the Decider does not handle it.",
        r.get("cmd").map(String::as_str).unwrap_or("?")),
};

/// §3.3 Output-alphabet containment — a Decider may only emit events that a
/// command it handles is declared to emit.
pub const OUTPUT_ALPHABET: SparqlRule = SparqlRule {
    id: "output-alphabet-containment",
    focus_var: "decider",
    path: "emits",
    severity: "violation",
    select: r#"
      SELECT ?decider ?event WHERE {
        ?decider a <https://productframework.org/ns#Decider> .
        ?decider <https://productframework.org/ns#emitsEvent> ?event .
        FILTER NOT EXISTS {
          ?decider <https://productframework.org/ns#handles> ?cmd .
          ?cmd <https://productframework.org/ns#emits> ?event .
        }
      }
    "#,
    message: |r| format!(
        "§3.3 Output-alphabet containment: a Decider emits '{}', which no handled command is declared to emit.",
        r.get("event").map(String::as_str).unwrap_or("?")),
};

/// The §3.3 conformance rules over a Decider + What projection.
pub fn decider_rules() -> &'static [SparqlRule] {
    &[DECIDES_FOR_ENTITY, NO_FOREIGN_COMMANDS, COMMAND_COVERAGE, OUTPUT_ALPHABET]
}

fn warn(focus: &str, path: &str, message: String) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message, severity: "warning".to_string() }
}

/// §3.3/§3.4 state + Decider justification — model-gap *detectors* (warnings,
/// peers to the intent-reliance and data-divergence signals, not blocking
/// gates). State justification: an aggregate field that is evolved but read by
/// no decision is dead state or an unmodelled invariant. Decider justification:
/// a Decider with no reachable rejection (no guard) is not deciding anything.
/// Only authored logic is judged — a signature-only Decider is a stub.
pub fn justification_findings(decider: &Decider) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(logic) = &decider.logic else { return out; };

    // Fields the aggregate evolves: initial state plus everything any event sets.
    let mut evolved: BTreeSet<String> = logic.initial.keys().cloned().collect();
    for ev in &logic.evolve {
        evolved.extend(ev.set.keys().cloned());
    }

    // Fields some decision reads: structured guards, the `reads` escape hatch,
    // and any field named in a CEL guard/emit expression (substring — err toward
    // "read" so a genuinely-used field is never wrongly flagged dead).
    let mut read: BTreeSet<String> = decider.reads.iter().cloned().collect();
    let mut exprs: Vec<String> = Vec::new();
    let mut guard_count = 0usize;
    for rule in &logic.decide {
        for g in &rule.guards {
            guard_count += 1;
            if let Some(p) = &g.when {
                read.insert(p.field.clone());
            }
            if let Some(e) = &g.expr {
                exprs.push(e.clone());
            }
        }
        for er in &rule.emit {
            for val in er.payload().values() {
                if let Scalar::Str(s) = val {
                    if s.starts_with('=') {
                        exprs.push(s.clone());
                    }
                }
            }
        }
    }

    // State justification: an evolved field nothing reads is a finding.
    for f in &evolved {
        let in_expr = exprs.iter().any(|e| e.contains(f.as_str()));
        if !read.contains(f) && !in_expr {
            out.push(warn(&decider.id, "reads", format!(
                "§3.3 State justification: aggregate field '{f}' is evolved but no decision reads it — dead state, or an unmodelled invariant (guard it, or list it in `reads`)."
            )));
        }
    }

    // Decider justification: a Decider that can never reject decides nothing.
    if guard_count == 0 {
        out.push(warn(&decider.id, "rejects",
            "§3.3 Decider justification: this Decider has no reachable rejection (no guard) — trivial behaviour that should have no Decider, or an unmodelled invariant.".to_string()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::sparql_rules::run_rules;
    use super::*;

    // A small What + Decider projection: aggregate Task, two commands target it.
    const PREFIXES: &str = "@prefix pf: <https://productframework.org/ns#> .\n@prefix d: <https://productframework.org/product/x#> .\n";
    const MODEL: &str = "d:Task a pf:Entity .\nd:CompleteTask a pf:Command ; pf:targets d:Task ; pf:emits d:Done .\nd:ReopenTask a pf:Command ; pf:targets d:Task ; pf:emits d:Reopened .\n";

    fn ttl(decider: &str) -> String {
        format!("{PREFIXES}{MODEL}{decider}")
    }

    #[test]
    fn full_conformant_decider_passes() {
        let d = "d:dec a pf:Decider ; pf:decidesFor d:Task ; pf:handles d:CompleteTask ; pf:handles d:ReopenTask ; pf:emitsEvent d:Done ; pf:emitsEvent d:Reopened .\n";
        assert!(run_rules(&ttl(d), decider_rules()).is_empty());
    }

    #[test]
    fn foreign_command_fires() {
        let d = "d:dec a pf:Decider ; pf:decidesFor d:Task ; pf:handles d:CompleteTask ; pf:handles d:ReopenTask ; pf:handles d:Alien ; pf:emitsEvent d:Done ; pf:emitsEvent d:Reopened .\n";
        let vs = run_rules(&ttl(d), &[NO_FOREIGN_COMMANDS]);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("Alien"));
    }

    #[test]
    fn missing_coverage_fires() {
        let d = "d:dec a pf:Decider ; pf:decidesFor d:Task ; pf:handles d:CompleteTask ; pf:emitsEvent d:Done .\n";
        let vs = run_rules(&ttl(d), &[COMMAND_COVERAGE]);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("ReopenTask"));
    }

    #[test]
    fn unsanctioned_event_fires() {
        let d = "d:dec a pf:Decider ; pf:decidesFor d:Task ; pf:handles d:CompleteTask ; pf:handles d:ReopenTask ; pf:emitsEvent d:Done ; pf:emitsEvent d:Reopened ; pf:emitsEvent d:Ghost .\n";
        let vs = run_rules(&ttl(d), &[OUTPUT_ALPHABET]);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("Ghost"));
    }

    #[test]
    fn deciding_for_a_non_entity_fires() {
        let d = "d:dec a pf:Decider ; pf:decidesFor d:Nope .\n";
        let vs = run_rules(&ttl(d), &[DECIDES_FOR_ENTITY]);
        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("Nope"));
    }

    use super::super::decider_logic::{DecideRule, DeciderLogic, EvolveRule, Guard, Predicate};

    fn logic_with(initial: &[(&str, bool)], guard_field: Option<&str>) -> DeciderLogic {
        let mut l = DeciderLogic::default();
        for (k, b) in initial {
            l.initial.insert((*k).into(), Scalar::Bool(*b));
        }
        l.evolve.push(EvolveRule { on: "Placed".into(), set: Default::default() });
        let guards = guard_field.map(|f| vec![Guard {
            when: Some(Predicate { field: f.into(), eq: Some(Scalar::Bool(false)), ne: None, any_of: None, exists: None }),
            expr: None, else_reject: "inv-1".into(),
        }]).unwrap_or_default();
        l.decide.push(DecideRule { on: "Place".into(), guards, emit: vec!["Placed".into()] });
        l
    }

    #[test]
    fn signature_only_decider_has_no_justification_findings() {
        let d = Decider { id: "d".into(), decides_for: "Task".into(), logic: None, ..Default::default() };
        assert!(justification_findings(&d).is_empty());
    }

    #[test]
    fn evolved_field_read_by_a_guard_is_justified() {
        // 'open' is both evolved (initial) and read by the guard → no finding.
        let d = Decider { id: "d".into(), decides_for: "Task".into(), logic: Some(logic_with(&[("open", true)], Some("open"))), ..Default::default() };
        assert!(justification_findings(&d).is_empty(), "{:?}", justification_findings(&d));
    }

    #[test]
    fn unread_evolved_field_and_toothless_decider_are_findings() {
        // 'open' is read; 'archived' is evolved but read by nothing → state finding.
        let mut logic = logic_with(&[("open", true), ("archived", false)], Some("open"));
        logic.initial.insert("archived".into(), Scalar::Bool(false));
        let d = Decider { id: "d".into(), decides_for: "Task".into(), logic: Some(logic), ..Default::default() };
        let vs = justification_findings(&d);
        assert!(vs.iter().any(|v| v.path == "reads" && v.message.contains("archived")), "{vs:?}");
        assert!(vs.iter().all(|v| v.severity == "warning"));

        // A decider with decide rules but no guard never rejects → decider finding.
        let d2 = Decider { id: "d2".into(), decides_for: "Task".into(), logic: Some(logic_with(&[], None)), ..Default::default() };
        let vs2 = justification_findings(&d2);
        assert!(vs2.iter().any(|v| v.path == "rejects"), "{vs2:?}");
    }

    #[test]
    fn reads_escape_hatch_justifies_a_cel_read_field() {
        // 'count' is evolved and listed in `reads` (a CEL guard reads it) → justified.
        let d = Decider {
            id: "d".into(), decides_for: "Task".into(), reads: vec!["count".into()],
            logic: Some(logic_with(&[("count", false)], Some("other"))), ..Default::default()
        };
        // 'other' is the guard field (not evolved), 'count' is evolved+in reads → no finding.
        assert!(justification_findings(&d).iter().all(|v| !v.message.contains("count")), "{:?}", justification_findings(&d));
    }
}
