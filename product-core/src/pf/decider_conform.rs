//! Behavioural conformance — realised code vs the Decider oracle (§6.3).
//!
//! The same scenarios the Decider was simulated against (§3.3) are replayed
//! against realised code: for each scenario the realised behaviour must produce
//! an outcome *identical* to the Decider's simulated outcome. This module is the
//! pure comparison plus the request shape; the CLI adapter spawns the runner and
//! feeds it the requests over a JSON protocol (array in, array out).

use serde::Serialize;

use super::decider::Decider;
use super::decider_logic::{CommandRef, EventRef, Expectation};
use super::decider_sim::{decide, replay, EmittedEvent, Outcome};
use super::validate::Violation;

/// One conformance request handed to the realised runner (one per scenario).
#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub given: Vec<EventRef>,
    pub when: CommandRef,
}

/// Build the runner requests, in scenario order.
pub fn requests(decider: &Decider) -> Vec<Request> {
    decider
        .scenarios
        .iter()
        .map(|s| Request { given: s.given.clone(), when: s.when.clone() })
        .collect()
}

/// Compare each realised response (parsed from the runner) to the Decider's
/// simulated outcome for that scenario. Empty = behaviourally conformant.
pub fn check_conformance(decider: &Decider, realised: &[Expectation]) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(logic) = &decider.logic else {
        out.push(v(&decider.id, "logic",
            "§6.3 A Decider needs logic + scenarios to check behavioural conformance."));
        return out;
    };
    if realised.len() != decider.scenarios.len() {
        out.push(v(&decider.id, "runner",
            &format!("§6.3 runner returned {} outcome(s) for {} scenario(s)", realised.len(), decider.scenarios.len())));
        return out;
    }
    for (s, got) in decider.scenarios.iter().zip(realised) {
        let oracle = match replay(logic, &s.given).and_then(|st| decide(logic, &st, &s.when)) {
            Ok(o) => o,
            Err(e) => {
                out.push(v(&s.name, "oracle", &format!("§6.3 oracle error for '{}': {e}", s.name)));
                continue;
            }
        };
        let realised_outcome = to_outcome(got);
        if oracle != realised_outcome {
            out.push(v(&s.name, "conformance",
                &format!("§6.3 realised behaviour differs for '{}': oracle {oracle:?}, realised {realised_outcome:?}", s.name)));
        }
    }
    out
}

/// Canonicalize a runner response into an `Outcome` (a `reject` wins if present).
fn to_outcome(resp: &Expectation) -> Outcome {
    if let Some(r) = &resp.reject {
        return Outcome::Rejected(r.clone());
    }
    let events = resp
        .emit
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|e: &EventRef| EmittedEvent { event: e.id().to_string(), payload: e.payload() })
        .collect();
    Outcome::Accepted(events)
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
#[path = "decider_conform_tests.rs"]
mod tests;
