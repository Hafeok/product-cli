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

// --- §4.6 content coverage ------------------------------------------------

/// §4.6 — content coverage + role conformance. Every (content key, locale) a UI
/// step references must be resolved by some content store in every locale the
/// stores claim; and an `error-message`/`empty-message` role must resolve to a
/// non-empty string. Roles tie the What-side meaning to the How-side value.
pub fn check_content_coverage(graph: &super::model::DomainGraph) -> Vec<super::validate::Violation> {
    let mut out = Vec::new();
    // The locales the application claims = union of every store's locales.
    let locales: std::collections::BTreeSet<&str> = graph
        .content_stores
        .iter()
        .flat_map(|s| s.locales.iter().map(String::as_str))
        .collect();
    if locales.is_empty() {
        return out; // no store declared yet — nothing to check against
    }
    let resolve = |key: &str, locale: &str| -> Option<&str> {
        graph.content_stores.iter().find_map(|s| {
            s.resolutions.iter().find(|r| r.key == key && r.locale == locale).map(|r| r.value.as_str())
        })
    };
    for step in &graph.wireframe_steps {
        for cref in &step.content_refs {
            for &loc in &locales {
                match resolve(&cref.key, loc) {
                    None => out.push(content_violation(&step.id, format!(
                        "§4.6 content '{}' is not resolved for locale '{loc}' (content coverage).",
                        cref.key
                    ))),
                    Some(v) if v.trim().is_empty() && is_message_role(&cref.role) => {
                        out.push(content_violation(&step.id, format!(
                            "§4.6 role '{}' for '{}' resolves to empty in '{loc}' (role conformance).",
                            cref.role, cref.key
                        )))
                    }
                    Some(_) => {}
                }
            }
        }
    }
    out
}

/// §3.2.1 — content on a UI step is a keyed *reference*, never a literal. A
/// content ref's key must be a key (no whitespace) with a declared role; a
/// literal sentence baked in as a "key" is rejected. Runs always (not gated on a
/// store), so the reference discipline holds before any store exists.
pub fn check_content_refs(graph: &super::model::DomainGraph) -> Vec<super::validate::Violation> {
    let mut out = Vec::new();
    for step in &graph.wireframe_steps {
        for cref in &step.content_refs {
            if cref.key.trim().is_empty() || cref.key.contains(char::is_whitespace) {
                out.push(content_violation(&step.id, format!(
                    "§3.2.1 content must be a keyed reference, not a literal — {:?} is not a content key.",
                    cref.key
                )));
            }
            if cref.role.trim().is_empty() {
                out.push(content_violation(&step.id, format!(
                    "§3.2.1 content reference '{}' must declare a role (heading/body/empty-message/…).",
                    cref.key
                )));
            }
        }
    }
    out
}

fn is_message_role(role: &str) -> bool {
    matches!(role, "error-message" | "empty-message" | "heading" | "legal")
}

fn content_violation(focus: &str, message: String) -> super::validate::Violation {
    super::validate::Violation {
        focus: focus.to_string(),
        path: "content_refs".to_string(),
        message,
        severity: "violation".to_string(),
    }
}

// --- §3.2.3 accessibility -------------------------------------------------

/// One WCAG criterion a step must satisfy, with where it came from and how it is
/// (or isn't) discharged.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Obligation {
    pub criterion: String,
    pub level: String,
    pub verification: String,
    /// "step" or the AIO id it was inherited from.
    pub source: String,
    pub discharged: bool,
    pub basis: String,
}

/// The accessibility verdict for one UI step: the computed obligation union and
/// whether each is discharged (machine gate satisfied, or an attestation
/// recorded). Reports level + basis, never a bare pass (§3.2.3).
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct AccessibilityVerdict {
    pub step: String,
    pub conformant: bool,
    pub obligations: Vec<Obligation>,
}

/// Look up a criterion's (level, verification) from the graph, then the built-in
/// CORE_WCAG seed.
fn criterion_meta(graph: &super::model::DomainGraph, id: &str) -> Option<(String, String)> {
    if let Some(c) = graph.wcag_criteria.iter().find(|c| c.id == id) {
        return Some((
            c.level.clone().unwrap_or_else(|| "A".into()),
            c.verification.clone().unwrap_or_else(|| "manual".into()),
        ));
    }
    super::ids::CORE_WCAG
        .iter()
        .find(|(cid, ..)| *cid == id)
        .map(|(_, level, vt, _)| ((*level).to_string(), (*vt).to_string()))
}

/// The criteria a step inherits from one AIO id (registered node or core seed).
fn aio_criteria(graph: &super::model::DomainGraph, aio: &str) -> Vec<String> {
    if let Some(node) = graph.aios.iter().find(|a| a.id == aio) {
        if !node.must_satisfy.is_empty() {
            return node.must_satisfy.clone();
        }
    }
    super::ids::CORE_AIO_CRITERIA
        .iter()
        .find(|(id, _)| *id == aio)
        .map(|(_, cs)| cs.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// Compute a step's accessibility verdict: the union of its AIOs' inherited
/// criteria plus its own `must_satisfy`, each discharged by a machine gate
/// (criterion `satisfied`) or a recorded attestation.
pub fn accessibility_verdict(graph: &super::model::DomainGraph, step_id: &str) -> Option<AccessibilityVerdict> {
    let step = graph.wireframe_steps.iter().find(|s| s.id == step_id)?;
    let mut obligations: Vec<Obligation> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut add = |criterion: &str, source: &str, obligations: &mut Vec<Obligation>| {
        if !seen.insert(criterion.to_string()) {
            return;
        }
        let (level, verification) = criterion_meta(graph, criterion)
            .unwrap_or_else(|| ("A".into(), "manual".into()));
        let satisfied = graph.wcag_criteria.iter().find(|c| c.id == criterion).map(|c| c.satisfied)
            .or_else(|| super::ids::CORE_WCAG.iter().find(|(id, ..)| *id == criterion).map(|_| false))
            .unwrap_or(false);
        let attested = graph.attestations.iter().any(|a| a.step == step_id && a.criterion == criterion);
        let (discharged, basis) = if verification == "machine" {
            if satisfied { (true, "machine gate".into()) } else { (false, "machine gate failed".into()) }
        } else if attested {
            (true, "attestation".into())
        } else {
            (false, "no attestation".into())
        };
        obligations.push(Obligation {
            criterion: criterion.to_string(), level, verification, source: source.to_string(), discharged, basis,
        });
    };

    // Inherited from the AIOs the step references (surfaces + offers), then own.
    for s in &step.surfaces {
        for c in aio_criteria(graph, &s.aio) { add(&c, &s.aio, &mut obligations); }
    }
    for o in &step.offers {
        for c in aio_criteria(graph, &o.aio) { add(&c, &o.aio, &mut obligations); }
    }
    for c in &step.must_satisfy { add(c, "step", &mut obligations); }

    let conformant = obligations.iter().all(|o| o.discharged);
    Some(AccessibilityVerdict { step: step_id.to_string(), conformant, obligations })
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
