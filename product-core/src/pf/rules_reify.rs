//! §4.5 reification checks — coverage, closed-vocabulary, tokens-not-literals.

use super::model::DomainGraph;
use super::validate::Violation;

fn violation(focus: &str, path: &str, message: String) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message, severity: "violation".to_string() }
}

/// The AIO ids any UI step references (through its surfaces/offers), deduped.
fn referenced_aios(graph: &DomainGraph) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    for step in &graph.wireframe_steps {
        set.extend(step.surfaces.iter().map(|s| s.aio.clone()));
        set.extend(step.offers.iter().map(|o| o.aio.clone()));
    }
    set.remove("");
    set
}

/// §4.5 reification coverage — every (AIO a step references, declared context of
/// use) pair must have a `reify(AIO, context) → CIO` rule, or some screen is left
/// unspecified for some device (the design-system analogue of command coverage).
pub fn check_reification_coverage(graph: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    let contexts: Vec<&str> = graph.contexts_of_use.iter().map(|c| c.id.as_str()).collect();
    if contexts.is_empty() {
        return out; // no contexts declared yet — nothing to cover
    }
    for aio in referenced_aios(graph) {
        for ctx in &contexts {
            let covered = graph
                .reification_rules
                .iter()
                .any(|r| r.aio == aio && r.context == *ctx);
            if !covered {
                out.push(violation(
                    &aio,
                    "reifies",
                    format!("§4.5 no reification rule for AIO '{aio}' in context '{ctx}' (reification coverage)."),
                ));
            }
        }
    }
    out
}

/// §4.5 closed vocabulary — a reification rule may only target a CIO the design
/// system defines (its catalog), never invent one.
pub fn check_closed_vocabulary(graph: &DomainGraph) -> Vec<Violation> {
    let catalog: std::collections::BTreeSet<&str> = graph
        .design_systems
        .iter()
        .flat_map(|d| d.cios.iter().map(String::as_str))
        .chain(graph.cios.iter().map(|c| c.id.as_str()))
        .collect();
    if catalog.is_empty() {
        return Vec::new();
    }
    graph
        .reification_rules
        .iter()
        .filter(|r| !catalog.contains(r.cio.as_str()))
        .map(|r| {
            violation(&r.id, "cio", format!(
                "§4.5 reification rule '{}' targets off-system component '{}' (not in the design-system catalog).",
                r.id, r.cio
            ))
        })
        .collect()
}

/// §4.5 tokens-not-literals — every style value a UI step carries must be a
/// declared design-system token, never a literal (e.g. `#3366ff`).
pub fn check_tokens_not_literals(graph: &DomainGraph) -> Vec<Violation> {
    let tokens: std::collections::BTreeSet<&str> = graph
        .design_systems
        .iter()
        .flat_map(|d| d.tokens.iter().map(String::as_str))
        .chain(graph.tokens.iter().map(|t| t.id.as_str()))
        .collect();
    let mut out = Vec::new();
    for step in &graph.wireframe_steps {
        for style in &step.styles {
            if !tokens.contains(style.as_str()) {
                out.push(violation(&step.id, "styles", format!(
                    "§4.5 style '{style}' is a literal, not a design-system token (tokens-not-literals)."
                )));
            }
        }
    }
    out
}

/// All §4.5 reification checks.
pub fn reify_checks(graph: &DomainGraph) -> Vec<Violation> {
    let mut v = check_reification_coverage(graph);
    v.extend(check_closed_vocabulary(graph));
    v.extend(check_tokens_not_literals(graph));
    v
}

#[cfg(test)]
#[path = "rules_reify_tests.rs"]
mod tests;
