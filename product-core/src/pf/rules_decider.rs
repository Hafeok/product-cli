//! SPARQL conformance rules for a Decider — the §3.3 anti-drift checks.
//!
//! The authored signature must not drift from the event model: it decides for a
//! real aggregate, handles no foreign commands, covers every command targeting
//! its aggregate, and emits only events its handled commands sanction. Each is a
//! SELECT over the combined What + Decider projection (`decider_to_turtle`),
//! returning one row per violation.

use super::sparql_rules::SparqlRule;

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
}
