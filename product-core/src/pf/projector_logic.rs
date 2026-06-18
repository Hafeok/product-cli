//! Authored fold logic for a Projector — pure declarative data (§3.4).
//!
//! A Projector's signature is derived from the event model; its logic is the one
//! authored part: how each event folds into the read model's view state. Unlike a
//! Decider there is no decide/guard half — a projection produces a view, not
//! events. Scenarios are the oracle: given a sequence of events, then the expected
//! view state. Reuses the Decider's value/event vocabulary (`Scalar`, `EventRef`,
//! `EvolveRule`, `State`); the interpreter lives in `projector_sim`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::decider_logic::{EventRef, EvolveRule, State};

/// The authored fold: an initial view and how each event updates it. `apply`
/// reuses the Decider's `EvolveRule` (`on` event → `set` fields, literal or CEL).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectorLogic {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub initial: State,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apply: Vec<EvolveRule>,
}

/// A projection scenario — the oracle, authored once and consumed twice
/// (pre-realisation simulation here; post-realisation conformance in §6.3): fold
/// the `given` events, expect the `then` view state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectorScenario {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub given: Vec<EventRef>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub then: State,
}
