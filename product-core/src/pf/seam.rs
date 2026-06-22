//! §6.3 — the seam verification. Composes the per-phase UI checks for one screen
//! (UI step) into a single verdict that reports level + basis, never a bare pass:
//! datum-projected, control-maps-to-command, AIO typing, state coverage,
//! content coverage, reification coverage, and accessibility discharge.

use super::model::DomainGraph;

/// One named sub-check of the seam, with its findings (empty = passed).
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct SeamCheck {
    pub name: String,
    pub passed: bool,
    pub findings: Vec<String>,
}

impl SeamCheck {
    fn new(name: &str, findings: Vec<String>) -> Self {
        SeamCheck { name: name.to_string(), passed: findings.is_empty(), findings }
    }
}

/// The composite seam verdict for one UI step.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct SeamVerdict {
    pub step: String,
    pub conformant: bool,
    pub checks: Vec<SeamCheck>,
}

/// Verify the seam for `step_id`, composing every per-phase check scoped to that
/// step. `None` if the step does not exist.
pub fn seam_verdict(graph: &DomainGraph, step_id: &str) -> Option<SeamVerdict> {
    let step = graph.wireframe_steps.iter().find(|s| s.id == step_id)?;
    let scope = |vs: Vec<super::validate::Violation>| -> Vec<String> {
        vs.into_iter().filter(|v| v.focus == step_id).map(|v| v.message).collect()
    };
    let mut content = scope(super::rules_ui::check_content_refs(graph));
    content.extend(scope(super::rules_ui::check_content_coverage(graph)));

    let checks = vec![
        SeamCheck::new("datum-projected", datum_check(graph, step)),
        SeamCheck::new("control-maps-to-command", control_check(graph, step)),
        SeamCheck::new("aio-typing", scope(super::sparql_rules::run_rules(
            &ui_projection(graph), super::rules_ui::ui_rules()))),
        SeamCheck::new("state-coverage", scope(super::rules_ui::check_state_coverage(graph))),
        SeamCheck::new("content-coverage", content),
        SeamCheck::new("reification-coverage", reification_check(graph, step)),
        SeamCheck::new("accessibility", accessibility_check(graph, step_id)),
    ];
    let conformant = checks.iter().all(|c| c.passed);
    Some(SeamVerdict { step: step_id.to_string(), conformant, checks })
}

/// datum-projected: every projection the page surfaces is supplied by a read model.
fn datum_check(graph: &DomainGraph, step: &super::model::WireframeStep) -> Vec<String> {
    step.surfaces
        .iter()
        .filter(|s| !graph.read_models.iter().any(|rm| rm.id == s.projection))
        .map(|s| format!("datum '{}' is not projected by any read model", s.projection))
        .collect()
}

/// control-maps-to-command: every control the page offers issues a real command.
fn control_check(graph: &DomainGraph, step: &super::model::WireframeStep) -> Vec<String> {
    step.offers
        .iter()
        .filter(|o| !graph.commands.iter().any(|c| c.id == o.command))
        .map(|o| format!("control offers '{}', which is not a command the step accepts", o.command))
        .collect()
}

/// Reification coverage is keyed by the offending AIO; scope it to this step's AIOs.
fn reification_check(graph: &DomainGraph, step: &super::model::WireframeStep) -> Vec<String> {
    let aios: std::collections::BTreeSet<&str> = step
        .surfaces
        .iter()
        .map(|s| s.aio.as_str())
        .chain(step.offers.iter().map(|o| o.aio.as_str()))
        .collect();
    super::rules_reify::check_reification_coverage(graph)
        .into_iter()
        .filter(|v| aios.contains(v.focus.as_str()))
        .map(|v| v.message)
        .collect()
}

/// Accessibility: the step's undischarged WCAG obligations (§3.2.3).
fn accessibility_check(graph: &DomainGraph, step_id: &str) -> Vec<String> {
    super::rules_ui::accessibility_verdict(graph, step_id)
        .map(|v| {
            v.obligations.iter().filter(|o| !o.discharged)
                .map(|o| format!("WCAG {} [{}] undischarged ({})", o.criterion, o.level, o.basis))
                .collect()
        })
        .unwrap_or_default()
}

/// The rules projection augmented with the core AIO vocabulary (mirrors
/// `validate::ui_projection`, kept local to avoid widening that module's API).
fn ui_projection(graph: &DomainGraph) -> String {
    let mut ttl = super::turtle::to_turtle(graph, "seam");
    ttl.push_str(&super::rules_ui::core_aio_triples());
    ttl
}

#[cfg(test)]
#[path = "seam_tests.rs"]
mod tests;
