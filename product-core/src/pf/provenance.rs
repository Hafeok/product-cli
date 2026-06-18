//! Provenance record for a finalized session.
//!
//! What lets the exported What graph be trusted as an authored artifact: who
//! was in the room, when, a content hash of the graph, the count of tool
//! calls that derived it (§5 of the spec).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The provenance returned by `session_finalize` alongside the Turtle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    pub product: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub participants: Vec<String>,
    /// Sha-256 of the exported Turtle, hex-encoded.
    pub content_hash: String,
    /// ISO-8601 UTC timestamp of finalization.
    pub finalized_at: String,
    /// Number of mutating tool calls in the session (the derivation length).
    pub tool_call_count: usize,
}

/// Hex-encode the SHA-256 of the canonical Turtle export.
pub fn content_hash(turtle: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(turtle.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_hex() {
        let h = content_hash("d:Task a pf:Entity .");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, content_hash("d:Task a pf:Entity ."));
        assert_ne!(h, content_hash("d:Other a pf:Entity ."));
    }
}
