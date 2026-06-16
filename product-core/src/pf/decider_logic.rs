//! Authored decision logic for a Decider — pure declarative data (§3.3).
//!
//! A Decider's *signature* is derived from the event model; its *logic* is the
//! one authored part: a guarded state machine. `evolve` folds events into a
//! small state; `decide` guards each command with predicates, each tied to the
//! invariant it protects, then emits the sanctioned events. Scenarios are the
//! oracle — given prior events, when a command, then events or a rejection.
//! This module is data only; the interpreter that runs it lives in `decider_sim`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A primitive value in aggregate state, command/event payloads, or a guard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scalar {
    Bool(bool),
    Int(i64),
    Str(String),
}

/// Aggregate state: named fields, deterministically ordered.
pub type State = BTreeMap<String, Scalar>;

/// A single structured guard predicate over the current state. Exactly one of
/// the operator fields is set (Stage 2 adds a CEL `expr` alternative).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Predicate {
    pub field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq: Option<Scalar>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ne: Option<Scalar>,
    #[serde(default, rename = "in", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Scalar>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exists: Option<bool>,
}

/// A precondition: when `when` fails, the command is rejected for `else_reject`
/// (an invariant id — the aggregate's invariant, now executable).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guard {
    pub when: Predicate,
    pub else_reject: String,
}

/// How one event evolves state: the fields it sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolveRule {
    pub on: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub set: State,
}

/// How one command decides: ordered guards, then the events it emits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecideRule {
    pub on: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<Guard>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emit: Vec<String>,
}

/// The authored guarded state machine.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeciderLogic {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub initial: State,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evolve: Vec<EvolveRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decide: Vec<DecideRule>,
}

/// The expected outcome of a scenario: events emitted, or a rejection. Exactly
/// one of `emit`/`reject` is set (a `reject` takes precedence when evaluated).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Expectation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emit: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject: Option<String>,
}

impl Expectation {
    /// Expect acceptance emitting exactly these events.
    pub fn emit(events: Vec<String>) -> Self {
        Self { emit: Some(events), reject: None }
    }

    /// Expect rejection for this invariant.
    pub fn reject(invariant: &str) -> Self {
        Self { emit: None, reject: Some(invariant.to_string()) }
    }
}

/// A behavioural scenario — the oracle, authored once and consumed twice
/// (pre-realisation simulation here; post-realisation conformance in §6.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub given: Vec<String>,
    pub when: String,
    pub then: Expectation,
}
