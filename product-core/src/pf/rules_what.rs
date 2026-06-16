//! SPARQL conformance rules for the What graph — structure, behaviour.
//!
//! Mirrors the load-bearing cross-references in `schema/shapes/shapes.shacl.ttl`
//! (§3.1/§3.2): an entity lives in a real context; an event changes a real
//! entity in a real context; a command targets a real entity, emitting at least
//! one real event. The presence/cardinality checks (non-empty definition,
//! cardinality, rationale, …) stay native in `validate`.

use super::sparql_rules::SparqlRule;

/// §3.1 An entity must belong to exactly one bounded context.
pub const ENTITY_IN_CONTEXT: SparqlRule = SparqlRule {
    id: "entity-in-context",
    focus_var: "e",
    path: "inContext",
    severity: "violation",
    select: r#"
      SELECT ?e WHERE {
        ?e a <https://productframework.org/ns#Entity> .
        FILTER NOT EXISTS {
          ?e <https://productframework.org/ns#inContext> ?c .
          ?c a <https://productframework.org/ns#BoundedContext> .
        }
      }
    "#,
    message: |_| "§3.1 An entity must belong to exactly one bounded context (never a flat model).".to_string(),
};

/// §3.2 Every event must change a real domain entity.
pub const EVENT_CHANGES_ENTITY: SparqlRule = SparqlRule {
    id: "event-changes-entity",
    focus_var: "ev",
    path: "changes",
    severity: "violation",
    select: r#"
      SELECT ?ev WHERE {
        ?ev a <https://productframework.org/ns#Event> .
        FILTER NOT EXISTS {
          ?ev <https://productframework.org/ns#changes> ?e .
          ?e a <https://productframework.org/ns#Entity> .
        }
      }
    "#,
    message: |_| "§3.2 Every event must change a real domain entity (the load-bearing relation; behaviour may not reference structure that does not exist).".to_string(),
};

/// §3.2 An event must live in a bounded context.
pub const EVENT_IN_CONTEXT: SparqlRule = SparqlRule {
    id: "event-in-context",
    focus_var: "ev",
    path: "inContext",
    severity: "violation",
    select: r#"
      SELECT ?ev WHERE {
        ?ev a <https://productframework.org/ns#Event> .
        FILTER NOT EXISTS {
          ?ev <https://productframework.org/ns#inContext> ?c .
          ?c a <https://productframework.org/ns#BoundedContext> .
        }
      }
    "#,
    message: |_| "§3.2 An event must live in a bounded context.".to_string(),
};

/// §3.2 A command must target a real aggregate (entity).
pub const COMMAND_TARGETS_ENTITY: SparqlRule = SparqlRule {
    id: "command-targets-entity",
    focus_var: "cmd",
    path: "targets",
    severity: "violation",
    select: r#"
      SELECT ?cmd WHERE {
        ?cmd a <https://productframework.org/ns#Command> .
        FILTER NOT EXISTS {
          ?cmd <https://productframework.org/ns#targets> ?e .
          ?e a <https://productframework.org/ns#Entity> .
        }
      }
    "#,
    message: |_| "§3.2 A command must target a real aggregate (entity).".to_string(),
};

/// §3.2 A command must emit at least one event (command coverage).
pub const COMMAND_EMITS_EVENT: SparqlRule = SparqlRule {
    id: "command-emits-event",
    focus_var: "cmd",
    path: "emits",
    severity: "violation",
    select: r#"
      SELECT ?cmd WHERE {
        ?cmd a <https://productframework.org/ns#Command> .
        FILTER NOT EXISTS {
          ?cmd <https://productframework.org/ns#emits> ?ev .
          ?ev a <https://productframework.org/ns#Event> .
        }
      }
    "#,
    message: |_| "§3.2 A command must emit at least one event (command coverage).".to_string(),
};

/// The What-graph cross-reference rules (§3.1 structure, §3.2 behaviour).
pub fn what_rules() -> &'static [SparqlRule] {
    &[
        ENTITY_IN_CONTEXT,
        EVENT_CHANGES_ENTITY,
        EVENT_IN_CONTEXT,
        COMMAND_TARGETS_ENTITY,
        COMMAND_EMITS_EVENT,
    ]
}

#[cfg(test)]
mod tests {
    use super::super::sparql_rules::run_rules;
    use super::*;

    const PREFIXES: &str = "@prefix pf: <https://productframework.org/ns#> .\n@prefix d: <https://productframework.org/product/x#> .\n";

    #[test]
    fn conformant_behaviour_graph_passes() {
        let ttl = format!("{PREFIXES}d:Tasks a pf:BoundedContext .\nd:Task a pf:Entity ; pf:inContext d:Tasks .\nd:Done a pf:Event ; pf:inContext d:Tasks ; pf:changes d:Task .\nd:Complete a pf:Command ; pf:inContext d:Tasks ; pf:targets d:Task ; pf:emits d:Done .\n");
        assert!(run_rules(&ttl, what_rules()).is_empty());
    }

    #[test]
    fn event_changing_a_non_entity_fires() {
        let ttl = format!("{PREFIXES}d:Tasks a pf:BoundedContext .\nd:Ghost a pf:Event ; pf:inContext d:Tasks ; pf:changes d:Nope .\n");
        let vs = run_rules(&ttl, what_rules());
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].path, "changes");
        assert_eq!(vs[0].focus, "Ghost");
    }

    #[test]
    fn command_emitting_a_non_event_fires() {
        let ttl = format!("{PREFIXES}d:Tasks a pf:BoundedContext .\nd:Task a pf:Entity ; pf:inContext d:Tasks .\nd:C a pf:Command ; pf:inContext d:Tasks ; pf:targets d:Task ; pf:emits d:Nope .\n");
        let vs = run_rules(&ttl, what_rules());
        assert!(vs.iter().any(|v| v.path == "emits" && v.focus == "C"));
    }

    #[test]
    fn entity_in_a_missing_context_fires() {
        let ttl = format!("{PREFIXES}d:Task a pf:Entity ; pf:inContext d:MissingCtx .\n");
        let vs = run_rules(&ttl, what_rules());
        assert!(vs.iter().any(|v| v.path == "inContext" && v.focus == "Task"));
    }
}
