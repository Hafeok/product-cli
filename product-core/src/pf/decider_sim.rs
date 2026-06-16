//! Decider simulation — the pure interpreter plus the scenario oracle (§3.3).
//!
//! `evolve`/`replay` fold events into state; `decide` evaluates a command's
//! guards (first failure rejects with its invariant) and otherwise emits the
//! sanctioned events. Every function is total and deterministic — the property
//! that lets a Decider be proven *sound and complete before any code exists*.
//! `simulate` runs the authored scenarios as the oracle: soundness (every
//! scenario's actual outcome matches its expectation) and completeness (every
//! handled command is exercised).

use super::decider::Decider;
use super::decider_logic::{DeciderLogic, Predicate, Scenario, State};
use super::validate::Violation;

/// The result of running a command through `decide`.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Accepted, emitting these event ids.
    Accepted(Vec<String>),
    /// Rejected for this reason (an invariant id, or a structural message).
    Rejected(String),
}

/// Fold a sequence of events into aggregate state from `initial`.
pub fn replay(logic: &DeciderLogic, events: &[String]) -> State {
    let mut state = logic.initial.clone();
    for ev in events {
        if let Some(rule) = logic.evolve.iter().find(|r| &r.on == ev) {
            for (k, v) in &rule.set {
                state.insert(k.clone(), v.clone());
            }
        }
    }
    state
}

/// Decide a command against the current state: the first failing guard rejects
/// with its invariant; otherwise the command's events are emitted.
pub fn decide(logic: &DeciderLogic, state: &State, command: &str) -> Outcome {
    let Some(rule) = logic.decide.iter().find(|r| r.on == command) else {
        return Outcome::Rejected(format!("no decide rule for command '{command}'"));
    };
    for g in &rule.guards {
        if !eval(&g.when, state) {
            return Outcome::Rejected(g.else_reject.clone());
        }
    }
    Outcome::Accepted(rule.emit.clone())
}

/// Evaluate a structured predicate against state. Total: a missing field makes
/// every comparison false (except `exists: false`).
fn eval(pred: &Predicate, state: &State) -> bool {
    let actual = state.get(&pred.field);
    if let Some(v) = &pred.eq {
        return actual == Some(v);
    }
    if let Some(v) = &pred.ne {
        return actual != Some(v);
    }
    if let Some(vs) = &pred.any_of {
        return actual.map(|a| vs.contains(a)).unwrap_or(false);
    }
    if let Some(must) = pred.exists {
        return actual.is_some() == must;
    }
    true
}

/// The outcome of one scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Run one scenario through the interpreter and compare to its expectation.
pub fn run_scenario(logic: &DeciderLogic, scenario: &Scenario) -> ScenarioResult {
    let state = replay(logic, &scenario.given);
    let outcome = decide(logic, &state, &scenario.when);
    let (passed, detail) = match (&scenario.then.reject, &scenario.then.emit, &outcome) {
        (Some(inv), _, Outcome::Rejected(actual)) => {
            (inv == actual, format!("expected reject '{inv}', got '{actual}'"))
        }
        (Some(inv), _, Outcome::Accepted(a)) => {
            (false, format!("expected reject '{inv}', but it accepted {a:?}"))
        }
        (None, Some(emit), Outcome::Accepted(actual)) => {
            (emit == actual, format!("expected emit {emit:?}, got {actual:?}"))
        }
        (None, Some(emit), Outcome::Rejected(r)) => {
            (false, format!("expected emit {emit:?}, but it rejected '{r}'"))
        }
        (None, None, _) => (false, "scenario declares neither emit nor reject".to_string()),
    };
    ScenarioResult {
        name: scenario.name.clone(),
        passed,
        detail: if passed { "ok".to_string() } else { detail },
    }
}

/// Simulate a Decider against its scenarios: soundness (every scenario passes)
/// and completeness (every handled command is exercised). Returns one
/// `Violation` per failure — empty means sound and complete.
pub fn simulate(decider: &Decider) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(logic) = &decider.logic else {
        out.push(v(&decider.id, "logic",
            "§3.3 A Decider needs authored logic + scenarios to be simulated."));
        return out;
    };
    if decider.scenarios.is_empty() {
        out.push(v(&decider.id, "scenarios",
            "§3.3 A Decider needs at least one scenario to prove its behaviour."));
    }
    for s in &decider.scenarios {
        if !decider.handles.contains(&s.when) {
            out.push(v(&s.name, "scenario",
                &format!("§3.3 scenario '{}' uses command '{}', which the Decider does not handle.", s.name, s.when)));
            continue;
        }
        let r = run_scenario(logic, s);
        if !r.passed {
            out.push(v(&s.name, "scenario", &format!("§3.3 scenario '{}' failed: {}", s.name, r.detail)));
        }
    }
    for cmd in &decider.handles {
        if !decider.scenarios.iter().any(|s| &s.when == cmd) {
            out.push(v(&decider.id, "completeness",
                &format!("§3.3 incomplete: command '{cmd}' has no scenario — behaviour is unspecified.")));
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
#[path = "decider_sim_tests.rs"]
mod tests;
