//! The domain state change binding's event model — act vocabulary + fact vocabulary.
//!
//! A port of what `check_resolution.py` reads: `context`, `facts` (the type
//! space) and `slices` (type + name, each with its read and write positions).
//! The file format is the binding's, used unchanged; this module adds only the
//! lookups the C# delta needs. It supplies no determinations.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

/// One fact in the vocabulary.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Fact {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One act instance: type plus name is the address every determination binds to.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Slice {
    #[serde(rename = "type")]
    pub act_type: String,
    pub name: String,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
}

impl Slice {
    /// `command:PlaceOrder` — the address as one string.
    pub fn address(&self) -> String {
        format!("{}:{}", self.act_type, self.name)
    }

    /// Every fact the act reads or writes.
    pub fn facts(&self) -> BTreeSet<&str> {
        self.reads.iter().chain(self.writes.iter()).map(String::as_str).collect()
    }
}

/// The event model of one context.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EventModel {
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub facts: Vec<Fact>,
    #[serde(default)]
    pub slices: Vec<Slice>,
}

/// Parse an event model from YAML.
pub fn load_event_model(text: &str) -> Result<EventModel> {
    let model: EventModel = serde_yaml::from_str(text)
        .map_err(|e| ProductError::ConfigError(format!("event model: {e}")))?;
    let mut seen = BTreeSet::new();
    for s in &model.slices {
        if !seen.insert(s.address()) {
            return Err(ProductError::ConfigError(format!(
                "event model names {} twice",
                s.address()
            )));
        }
    }
    Ok(model)
}

impl EventModel {
    /// Is `fact` in the vocabulary?
    pub fn has_fact(&self, fact: &str) -> bool {
        self.facts.iter().any(|f| f.id == fact)
    }

    /// The act instances named `name`, whatever their type.
    pub fn slices_named(&self, name: &str) -> Vec<&Slice> {
        self.slices.iter().filter(|s| s.name == name).collect()
    }

    /// Every act whose read or write positions include `fact`.
    pub fn acts_touching(&self, fact: &str) -> Vec<&Slice> {
        self.slices.iter().filter(|s| s.facts().contains(fact)).collect()
    }

    /// Fact → the acts touching it, for every fact in the vocabulary.
    pub fn fact_index(&self) -> BTreeMap<&str, Vec<&Slice>> {
        self.facts.iter().map(|f| (f.id.as_str(), self.acts_touching(&f.id))).collect()
    }
}

#[cfg(test)]
#[path = "eventmodel_tests.rs"]
mod tests;
