//! Named-algorithm primitive — the Polanyi floor (§3.5).
//!
//! Some computations don't reduce to declared rules over a generic interpreter
//! (betweenness centrality, SHA-256, a YAML grammar). The honest move is to name
//! and bound them, not pretend they're derivable: a Primitive is specified by
//! *reference* (the named algorithm/standard — the name is the specification),
//! plus an I/O contract in the domain model's terms, plus an *oracle* (reference
//! input/output pairs). It is exempt from derivation and behavioural simulation,
//! and checked only by **oracle conformance** (§6.3) — its realised output must
//! match the reference algorithm's across the oracle pairs.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// One reference input/output pair that pins the primitive's behaviour.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OraclePair {
    pub input: String,
    pub output: String,
}

/// A §3.5 named-algorithm primitive — specified by reference, pinned by oracle.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Primitive {
    pub id: String,
    /// The named algorithm or standard it implements (the specification itself).
    pub reference: String,
    /// I/O contract — input and output types, in the domain model's terms.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub input: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub output: String,
    /// The reference input/output pairs — the only available check (§3.5).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oracle: Vec<OraclePair>,
}

impl Primitive {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid primitive YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize primitive: {}", e)))
    }
}

/// Validate a named-algorithm primitive (§3.5): it must name what it implements,
/// declare an I/O contract, and carry at least one oracle pair — because the
/// oracle is the *only* check it gets (it has no Decider or Projector to simulate).
pub fn validate_primitive(p: &Primitive) -> Vec<Violation> {
    let mut out = Vec::new();
    if p.id.trim().is_empty() {
        out.push(v(&p.id, "id", "§3.5 A named-algorithm primitive must be named."));
    }
    if p.reference.trim().is_empty() {
        out.push(v(&p.id, "reference", "§3.5 A primitive must name the algorithm/standard it implements — the name is the specification."));
    }
    if p.input.trim().is_empty() || p.output.trim().is_empty() {
        out.push(v(&p.id, "io", "§3.5 A primitive must declare an I/O contract (input and output types)."));
    }
    if p.oracle.is_empty() {
        out.push(v(&p.id, "oracle", "§3.5 A primitive must be pinned by at least one oracle pair — it has no other check."));
    }
    out
}

/// Oracle conformance (§6.3): the realised outputs (one per oracle pair, in
/// order) must match the reference algorithm's. Empty result = conformant.
pub fn check_oracle(p: &Primitive, realised: &[String]) -> Vec<Violation> {
    let mut out = Vec::new();
    if realised.len() != p.oracle.len() {
        out.push(v(&p.id, "oracle",
            &format!("§3.5 runner produced {} output(s) for {} oracle pair(s)", realised.len(), p.oracle.len())));
        return out;
    }
    for (i, (pair, got)) in p.oracle.iter().zip(realised).enumerate() {
        if &pair.output != got {
            out.push(v(&p.id, "oracle",
                &format!("§3.5 oracle pair {i} ('{}'): expected '{}', got '{}'", pair.input, pair.output, got)));
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
#[path = "primitive_tests.rs"]
mod tests;
