//! Strict What-conformance — the graph-level completeness checks (§3.2.0, §3.2.5,
//! §3.4, §4.5) that judge the whole model rather than one mutating node.
//!
//! These are the spec's *verification kinds*, not per-mutation gates: they ask
//! whether the captured What is complete (every flow owned, every command
//! triggered and event-producing, every view consumed, no step stranded on an
//! unreifiable AIO). They run only under `domain validate --strict`, so ordinary
//! authoring is never blocked by an in-progress graph.

use std::collections::BTreeSet;

use super::model::DomainGraph;
use super::validate::Violation;

fn finding(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

/// Run every strict completeness check over the whole graph.
pub fn pattern_conformance(graph: &DomainGraph) -> Vec<Violation> {
    let mut v = Vec::new();
    flow_ownership(graph, &mut v);
    command_pattern(graph, &mut v);
    view_consumed(graph, &mut v);
    unreifiable_seam(graph, &mut v);
    v
}

/// §3.2.5 — a flow belongs to exactly one system. Strict: every flow names one.
fn flow_ownership(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for f in &graph.flows {
        if f.system.is_none() {
            v.push(finding(&f.id, "system",
                "§3.2.5 Flow ownership: this flow belongs to no system — every flow belongs to exactly one."));
        }
    }
}

/// §3.2.0 — the Command pattern is `Trigger → Command → Event(s)`: a command
/// traces back to a Trigger and produces at least one event.
fn command_pattern(graph: &DomainGraph, v: &mut Vec<Violation>) {
    let issued: BTreeSet<&str> = graph.triggers.iter().map(|t| t.issues.as_str()).collect();
    for c in &graph.commands {
        if c.emits.is_empty() {
            v.push(finding(&c.id, "emits",
                "§3.2.0 Command pattern: a command must produce at least one event."));
        }
        if !issued.contains(c.id.as_str()) {
            v.push(finding(&c.id, "trigger",
                "§3.2.0 Command pattern: a command must trace to a Trigger that issues it."));
        }
    }
}

/// §3.4 — the projection mirror: a read model (View) earns its place only when
/// something consumes it — a UI step that surfaces it, or an automation that
/// watches it. An unconsumed View is a finding (the read-side of state minimality).
fn view_consumed(graph: &DomainGraph, v: &mut Vec<Violation>) {
    let mut consumed: BTreeSet<&str> = BTreeSet::new();
    for w in &graph.wireframe_steps {
        for s in &w.surfaces {
            consumed.insert(s.projection.as_str());
        }
        if let Some(d) = &w.displays {
            consumed.insert(d.as_str());
        }
    }
    for t in &graph.triggers {
        if let Some(w) = &t.watches {
            consumed.insert(w.as_str());
        }
    }
    for rm in &graph.read_models {
        if !consumed.contains(rm.id.as_str()) {
            v.push(finding(&rm.id, "consumed",
                "§3.4 View consumed: no UI step surfaces and no automation watches this read model — an unconsumed projection."));
        }
    }
}

/// §4.5 — the unreifiable seam: a UI step may not use an AIO that is declared
/// unreifiable in an interaction class one of the system's targets, or the step
/// cannot run in that class. A declared-unreifiable pair is the *recorded* gap;
/// using it where a system targets the class is the finding.
fn unreifiable_seam(graph: &DomainGraph, v: &mut Vec<Violation>) {
    for u in &graph.unreifiable_rules {
        let class_targeted = graph.systems.iter().any(|s| s.target_classes.iter().any(|c| c == &u.class));
        if !class_targeted {
            continue;
        }
        for w in &graph.wireframe_steps {
            let uses = w.surfaces.iter().any(|s| s.aio == u.aio) || w.offers.iter().any(|o| o.aio == u.aio);
            if uses {
                v.push(finding(&w.id, "aio", &format!(
                    "§4.5 Unreifiable seam: this UI step uses AIO '{}', unreifiable in class '{}' which a system targets.",
                    u.aio, u.class)));
            }
        }
    }
}

#[cfg(test)]
#[path = "rules_pattern_tests.rs"]
mod tests;
