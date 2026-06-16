//! Authored decision logic for a Decider — pure declarative data (§3.3).
//!
//! A Decider's *signature* is derived from the event model; its *logic* is the
//! one authored part: a guarded state machine. `evolve` folds events into a
//! small state; `decide` guards each command with predicates, each tied to the
//! invariant it protects, then emits the sanctioned events. Scenarios are the
//! oracle — given prior events, when a command, then events or a rejection.
//! Guards may be structured predicates or CEL expressions; events and commands
//! may carry payloads. This module is data only; the interpreter lives in
//! `decider_sim` and the CEL layer in `decider_cel`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A primitive value in aggregate state, payloads, or a guard. A string with a
/// leading `=` in an assignment position is a CEL expression (see `decider_cel`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scalar {
    Bool(bool),
    Int(i64),
    Str(String),
}

/// Aggregate state: named fields, deterministically ordered.
pub type State = BTreeMap<String, Scalar>;

/// A command/event payload — concrete in scenarios, expression-valued in logic.
pub type Payload = BTreeMap<String, Scalar>;

/// A reference to an event, optionally carrying a payload. Deserializes from a
/// bare id string or a `{event, with}` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventRef {
    Id(String),
    Data {
        event: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        with: Payload,
    },
}

impl EventRef {
    pub fn id(&self) -> &str {
        match self {
            EventRef::Id(s) => s,
            EventRef::Data { event, .. } => event,
        }
    }
    pub fn payload(&self) -> Payload {
        match self {
            EventRef::Id(_) => Payload::new(),
            EventRef::Data { with, .. } => with.clone(),
        }
    }
}

impl From<&str> for EventRef {
    fn from(s: &str) -> Self {
        EventRef::Id(s.to_string())
    }
}
impl From<String> for EventRef {
    fn from(s: String) -> Self {
        EventRef::Id(s)
    }
}

/// A reference to a command, optionally carrying a payload. Deserializes from a
/// bare id string or a `{command, with}` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandRef {
    Id(String),
    Data {
        command: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        with: Payload,
    },
}

impl CommandRef {
    pub fn id(&self) -> &str {
        match self {
            CommandRef::Id(s) => s,
            CommandRef::Data { command, .. } => command,
        }
    }
    pub fn payload(&self) -> Payload {
        match self {
            CommandRef::Id(_) => Payload::new(),
            CommandRef::Data { with, .. } => with.clone(),
        }
    }
}

impl From<&str> for CommandRef {
    fn from(s: &str) -> Self {
        CommandRef::Id(s.to_string())
    }
}

/// A single structured guard predicate over a field. Exactly one operator is set.
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

/// A precondition: when it fails, the command is rejected for `else_reject` (an
/// invariant id). The condition is a structured `when` predicate or a CEL `expr`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guard {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Predicate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expr: Option<String>,
    pub else_reject: String,
}

/// How one event evolves state: the fields it sets (literal or `=` CEL).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolveRule {
    pub on: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub set: BTreeMap<String, Scalar>,
}

/// How one command decides: ordered guards, then the events it emits (each
/// optionally with a payload whose values may be `=` CEL expressions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecideRule {
    pub on: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<Guard>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emit: Vec<EventRef>,
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
    pub emit: Option<Vec<EventRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject: Option<String>,
}

impl Expectation {
    /// Expect acceptance emitting exactly these events.
    pub fn emit(events: Vec<EventRef>) -> Self {
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
    pub given: Vec<EventRef>,
    pub when: CommandRef,
    pub then: Expectation,
}
