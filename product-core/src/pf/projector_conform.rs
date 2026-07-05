//! Projection conformance — realised read-side code vs the Projector oracle.
//!
//! The §3.4 mirror of `decider_conform`: the same scenarios the Projector
//! was simulated against are replayed against realised code. For each
//! scenario the runner folds the `given` events and answers the resulting
//! view state; the realised view must equal the oracle's fold **exactly**
//! (full-state equality, matching `projector_sim::simulate`). This module
//! is the pure comparison plus the request shape; the CLI adapter spawns
//! the runner over the same JSON protocol (array in, array out).

use serde::Serialize;

use super::decider_logic::{EventRef, State};
use super::projector::Projector;
use super::projector_sim::project;
use super::validate::Violation;

/// One projection request handed to the realised runner (one per scenario).
#[derive(Debug, Clone, Serialize)]
pub struct Request {
    pub given: Vec<EventRef>,
}

/// Build the runner requests, in scenario order.
pub fn requests(projector: &Projector) -> Vec<Request> {
    projector.scenarios.iter().map(|s| Request { given: s.given.clone() }).collect()
}

/// Compare each realised view (parsed from the runner) to the oracle's fold
/// for that scenario. Empty = behaviourally conformant.
pub fn check_conformance(projector: &Projector, realised: &[State]) -> Vec<Violation> {
    let mut out = Vec::new();
    let Some(logic) = &projector.logic else {
        out.push(v(&projector.id, "logic",
            "§3.4 A Projector needs logic + scenarios to check projection conformance."));
        return out;
    };
    if realised.len() != projector.scenarios.len() {
        out.push(v(&projector.id, "runner",
            &format!("§3.4 runner returned {} view(s) for {} scenario(s)", realised.len(), projector.scenarios.len())));
        return out;
    }
    for (s, got) in projector.scenarios.iter().zip(realised) {
        let oracle = match project(logic, &s.given) {
            Ok(state) => state,
            Err(e) => {
                out.push(v(&s.name, "oracle", &format!("§3.4 oracle error for '{}': {e}", s.name)));
                continue;
            }
        };
        if oracle != *got {
            out.push(v(&s.name, "conformance",
                &format!("§3.4 realised view differs for '{}': oracle {oracle:?}, realised {got:?}", s.name)));
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
#[path = "projector_conform_tests.rs"]
mod tests;
