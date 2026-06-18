//! Build verification planning — the §6 gate that runs each acceptance runner.
//!
//! Pure planning only: turns a deliverable's acceptance criteria into the
//! concrete commands a verifier executes. The adapter runs them and records the
//! pass/fail verdict back into each criterion's `status` (which `feature_done`
//! consumes), so verification stays a recorded predicate, never a judgement.

use super::deliverable::{AcceptanceCriterion, Deliverable};

/// One planned verification: the criterion it checks and the command to run.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifyStep {
    /// The acceptance criterion id this step records a verdict for.
    pub criterion: String,
    /// The program to spawn.
    pub program: String,
    /// Its arguments.
    pub args: Vec<String>,
}

/// Plan the runnable verifications for a deliverable. Criteria without a known
/// `runner` are skipped — they are judged manually and `build --verify` leaves
/// their status untouched.
pub fn plan(d: &Deliverable) -> Vec<VerifyStep> {
    d.acceptance.iter().filter_map(step_for).collect()
}

/// The criterion ids that carry a runner but name an unknown one — surfaced so a
/// verifier can warn rather than silently skip them.
pub fn unknown_runners(d: &Deliverable) -> Vec<String> {
    d.acceptance
        .iter()
        .filter(|a| a.runner.as_deref().is_some_and(|r| r != "cargo-test" && r != "shell"))
        .map(|a| a.id.clone())
        .collect()
}

fn step_for(a: &AcceptanceCriterion) -> Option<VerifyStep> {
    let runner = a.runner.as_deref()?;
    let raw = a.runner_args.clone().unwrap_or_default();
    let (program, args) = match runner {
        "cargo-test" => {
            let mut args = vec!["test".to_string()];
            args.extend(raw.split_whitespace().map(String::from));
            ("cargo".to_string(), args)
        }
        "shell" => ("sh".to_string(), vec!["-c".to_string(), raw]),
        _ => return None,
    };
    Some(VerifyStep { criterion: a.id.clone(), program, args })
}

#[cfg(test)]
#[path = "verify_tests.rs"]
mod tests;
