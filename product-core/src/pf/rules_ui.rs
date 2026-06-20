//! SPARQL conformance rules for the UI layer (§3.2.1) — the structural
//! AIO-only boundary.
//!
//! A UI step's interactions may reference only Abstract Interaction Objects
//! (§3.2.2), never a concrete control (a CIO) from a design system. Because the
//! closed-core AIO vocabulary (`ids::CORE_AIOS`) is recognised but not stored as
//! nodes, [`core_aio_triples`] injects it into the projection the rule runs over.

use super::sparql_rules::SparqlRule;

/// §3.2.1 — a UI step's interactions must be typed against an AIO, never a CIO.
pub const UI_INTERACTION_TYPED_AS_AIO: SparqlRule = SparqlRule {
    id: "ui-interaction-typed-as-aio",
    focus_var: "step",
    path: "typedAs",
    severity: "violation",
    select: r#"
      SELECT ?step WHERE {
        ?step a <https://productframework.org/ns#WireframeStep> ;
              <https://productframework.org/ns#typedAs> ?aio .
        FILTER NOT EXISTS { ?aio a <https://productframework.org/ns#Aio> . }
      }
    "#,
    message: |_| "§3.2.1 A UI step's interactions must be typed against an Abstract Interaction Object (an AIO), never a concrete control (a CIO).".to_string(),
};

/// The UI-layer cross-reference rules (§3.2.1).
pub fn ui_rules() -> &'static [SparqlRule] {
    &[UI_INTERACTION_TYPED_AS_AIO]
}

/// Turtle triples declaring the closed-core AIO vocabulary as `pf:Aio`, so the
/// AIO-only rule recognises the built-in set. Appended to the rules projection.
pub fn core_aio_triples() -> String {
    super::ids::CORE_AIOS
        .iter()
        .map(|a| format!("d:{a} a pf:Aio .\n"))
        .collect()
}

/// §3.2.1 — UI state coverage (the UI analogue of Decider command coverage). For
/// every surfaced projection, a UI step's state annotations must be **covering**
/// (a meaning or a waiver for every state in the projection's state space —
/// `present` plus its declared `loading`/`empty`/`failed`) and **constrained**
/// (no meaning for a state the projection cannot exhibit). Native, not SPARQL:
/// exhaustiveness over a per-projection alphabet.
pub fn check_state_coverage(graph: &super::model::DomainGraph) -> Vec<super::validate::Violation> {
    let mut out = Vec::new();
    for step in &graph.wireframe_steps {
        for surface in &step.surfaces {
            let Some(rm) = graph.read_models.iter().find(|r| r.id == surface.projection) else {
                continue;
            };
            // `present` is always a valid annotation but is the default and not
            // required; coverage targets the forgettable declared states.
            let required: Vec<&str> = rm.states.iter().map(String::as_str).collect();
            let valid: std::collections::HashSet<&str> =
                required.iter().copied().chain(["present"]).collect();
            let annotated = |state: &str| {
                step.state_meanings.iter().any(|m| {
                    m.projection == surface.projection
                        && m.state == state
                        && (m.meaning.is_some() || m.waiver.is_some())
                })
            };
            for s in &required {
                if !annotated(s) {
                    out.push(violation(&step.id, format!(
                        "§3.2 state '{s}' of projection '{}' has no meaning or waiver (state coverage).",
                        surface.projection
                    )));
                }
            }
            for m in step.state_meanings.iter().filter(|m| m.projection == surface.projection) {
                if !valid.contains(m.state.as_str()) {
                    out.push(violation(&step.id, format!(
                        "§3.2 state '{}' is not in projection '{}' state space (constrained).",
                        m.state, surface.projection
                    )));
                }
            }
        }
    }
    out
}

fn violation(focus: &str, message: String) -> super::validate::Violation {
    super::validate::Violation {
        focus: focus.to_string(),
        path: "state_meanings".to_string(),
        message,
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::sparql_rules::run_rules;
    use super::*;

    fn ttl(body: &str) -> String {
        format!(
            "@prefix pf: <https://productframework.org/ns#> .\n@prefix d: <https://productframework.org/product/x#> .\n{}{}",
            core_aio_triples(),
            body
        )
    }

    #[test]
    fn step_typed_against_a_core_aio_passes() {
        let g = ttl("d:Review a pf:WireframeStep ; pf:offers d:Confirm ; pf:typedAs d:trigger-action .\n");
        assert!(run_rules(&g, ui_rules()).is_empty());
    }

    #[test]
    fn step_typed_against_a_registered_aio_passes() {
        let g = ttl("d:RangeSel a pf:Aio .\nd:Review a pf:WireframeStep ; pf:typedAs d:RangeSel .\n");
        assert!(run_rules(&g, ui_rules()).is_empty());
    }

    #[test]
    fn step_referencing_a_cio_fires() {
        let g = ttl("d:Review a pf:WireframeStep ; pf:offers d:Confirm ; pf:typedAs d:primary-button .\n");
        let vs = run_rules(&g, ui_rules());
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].path, "typedAs");
        assert_eq!(vs[0].focus, "Review");
    }
}
