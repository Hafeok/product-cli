//! Decider simulation — the pure interpreter plus the scenario oracle (§3.3).
//!
//! `evolve`/`replay` fold events (with payloads) into state; `decide` evaluates
//! a command's guards (structured or CEL; first failure rejects with its
//! invariant) and otherwise emits its events with computed payloads. Every
//! function is total and deterministic — the property that lets a Decider be
//! proven *sound and complete before any code exists*. `simulate` runs the
//! authored scenarios as the oracle: soundness (every scenario's outcome matches
//! its expectation) and completeness (every handled command is exercised).

use super::decider::Decider;
use super::decider_cel::{eval_bool, eval_value};
use super::decider_logic::{
    CommandRef, DeciderLogic, EventRef, Guard, Payload, Predicate, Scenario, State,
};
use super::validate::Violation;

/// An emitted event with its computed payload.
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedEvent {
    pub event: String,
    pub payload: Payload,
}

/// The result of running a command through `decide`.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Accepted, emitting these events.
    Accepted(Vec<EmittedEvent>),
    /// Rejected for this reason (an invariant id, or a structural message).
    Rejected(String),
}

/// Fold a sequence of events into aggregate state from `initial`. Each event's
/// `set` values are evaluated against `{state, event}` (literal or `=` CEL).
pub fn replay(logic: &DeciderLogic, given: &[EventRef]) -> Result<State, String> {
    let mut state = logic.initial.clone();
    for ev in given {
        let Some(rule) = logic.evolve.iter().find(|r| r.on == ev.id()) else {
            continue;
        };
        let payload = ev.payload();
        let bindings = [("state", &state), ("event", &payload)];
        let mut next = state.clone();
        for (field, value) in &rule.set {
            next.insert(field.clone(), eval_value(value, &bindings)?);
        }
        state = next;
    }
    Ok(state)
}

/// Decide a command against the current state: the first failing guard rejects
/// with its invariant; otherwise the command's events are emitted with payloads.
pub fn decide(logic: &DeciderLogic, state: &State, command: &CommandRef) -> Result<Outcome, String> {
    let Some(rule) = logic.decide.iter().find(|r| r.on == command.id()) else {
        return Ok(Outcome::Rejected(format!("no decide rule for command '{}'", command.id())));
    };
    let cmd_payload = command.payload();
    let bindings = [("state", state), ("command", &cmd_payload)];
    for g in &rule.guards {
        if !guard_holds(g, &bindings) {
            return Ok(Outcome::Rejected(g.else_reject.clone()));
        }
    }
    let mut emitted = Vec::new();
    for spec in &rule.emit {
        let mut payload = Payload::new();
        for (field, value) in spec.payload() {
            payload.insert(field, eval_value(&value, &bindings)?);
        }
        emitted.push(EmittedEvent { event: spec.id().to_string(), payload });
    }
    Ok(Outcome::Accepted(emitted))
}

/// Evaluate a guard — a CEL expression if present, else a structured predicate.
fn guard_holds(g: &Guard, bindings: &[(&str, &Payload)]) -> bool {
    if let Some(expr) = &g.expr {
        return eval_bool(expr, bindings);
    }
    match &g.when {
        Some(pred) => eval_predicate(pred, bindings),
        None => true,
    }
}

/// Evaluate a structured predicate against the `state` binding. Total: a missing
/// field makes every comparison false (except `exists: false`).
fn eval_predicate(pred: &Predicate, bindings: &[(&str, &Payload)]) -> bool {
    let state = bindings.iter().find(|(n, _)| *n == "state").map(|(_, m)| *m);
    let actual = state.and_then(|s| s.get(&pred.field));
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
    let outcome = replay(logic, &scenario.given)
        .and_then(|state| decide(logic, &state, &scenario.when));
    let (passed, detail) = match outcome {
        Err(e) => (false, format!("logic error: {e}")),
        Ok(outcome) => compare(scenario, &outcome),
    };
    ScenarioResult { name: scenario.name.clone(), passed, detail }
}

fn compare(scenario: &Scenario, outcome: &Outcome) -> (bool, String) {
    match (&scenario.then.reject, &scenario.then.emit, outcome) {
        (Some(inv), _, Outcome::Rejected(actual)) => {
            (inv == actual, format!("expected reject '{inv}', got '{actual}'"))
        }
        (Some(inv), _, Outcome::Accepted(a)) => {
            (false, format!("expected reject '{inv}', but it accepted {a:?}"))
        }
        (None, Some(emit), Outcome::Accepted(actual)) => {
            let expected = expected_events(emit);
            (expected == *actual, format!("expected emit {expected:?}, got {actual:?}"))
        }
        (None, Some(emit), Outcome::Rejected(r)) => {
            (false, format!("expected emit {:?}, but it rejected '{r}'", expected_events(emit)))
        }
        (None, None, _) => (false, "scenario declares neither emit nor reject".to_string()),
    }
}

fn expected_events(refs: &[EventRef]) -> Vec<EmittedEvent> {
    refs.iter()
        .map(|r| EmittedEvent { event: r.id().to_string(), payload: r.payload() })
        .collect()
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
        if !decider.handles.iter().any(|h| h == s.when.id()) {
            out.push(v(&s.name, "scenario",
                &format!("§3.3 scenario '{}' uses command '{}', which the Decider does not handle.", s.name, s.when.id())));
            continue;
        }
        let r = run_scenario(logic, s);
        if !r.passed {
            out.push(v(&s.name, "scenario", &format!("§3.3 scenario '{}' failed: {}", s.name, r.detail)));
        }
    }
    for cmd in &decider.handles {
        if !decider.scenarios.iter().any(|s| s.when.id() == cmd) {
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
