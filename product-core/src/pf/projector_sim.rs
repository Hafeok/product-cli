//! Pure projection interpreter + scenario oracle (§3.4).
//!
//! Folds an event sequence through a Projector's authored logic into a view, then
//! proves the Projector sound (each scenario's folded view matches its expected
//! state) and complete (every folded event is exercised by a scenario) — before
//! any realisation, mirroring the Decider simulation. Pure and deterministic.

use super::decider_cel::eval_value;
use super::decider_logic::{EventRef, State};
use super::projector::Projector;
use super::projector_logic::ProjectorLogic;
use super::validate::Violation;

/// Fold the `given` events through the logic into a view state. A matched
/// `apply` rule's `set` values are evaluated against `{view, event}` bindings.
pub fn project(logic: &ProjectorLogic, given: &[EventRef]) -> Result<State, String> {
    let mut state = logic.initial.clone();
    for ev in given {
        let Some(rule) = logic.apply.iter().find(|r| r.on == ev.id()) else {
            continue;
        };
        let snapshot = state.clone();
        let payload = ev.payload();
        for (k, val) in &rule.set {
            let computed = eval_value(val, &[("view", &snapshot), ("event", &payload)])?;
            state.insert(k.clone(), computed);
        }
    }
    Ok(state)
}

/// Simulate a Projector's scenarios — an empty result means **sound and
/// complete**: every scenario's folded view matches its expectation, and every
/// folded event is exercised by at least one scenario.
pub fn simulate(projector: &Projector) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(logic) = &projector.logic else {
        out.push(v(&projector.id, "logic", "§3.4 A Projector needs authored logic + scenarios to be simulated."));
        return out;
    };
    if projector.scenarios.is_empty() {
        out.push(v(&projector.id, "scenarios", "§3.4 A Projector needs at least one scenario to prove its projection."));
    }
    for s in &projector.scenarios {
        match project(logic, &s.given) {
            Ok(state) if state == s.then => {}
            Ok(state) => out.push(v(&s.name, "scenario",
                &format!("§3.4 scenario '{}' failed: expected {:?}, projected {:?}", s.name, s.then, state))),
            Err(e) => out.push(v(&s.name, "scenario", &format!("§3.4 scenario '{}' errored: {e}", s.name))),
        }
    }
    for ev in &projector.folds {
        if !projector.scenarios.iter().any(|s| s.given.iter().any(|g| g.id() == ev)) {
            out.push(v(&projector.id, "completeness",
                &format!("§3.4 incomplete: event '{ev}' is folded but no scenario exercises it.")));
        }
    }
    out
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
#[path = "projector_sim_tests.rs"]
mod tests;
