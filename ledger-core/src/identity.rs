//! Actor identity: the claimed email an acceptance names (OD-3, resolved).
//!
//! `accepted-by` is a **claimed** identity sourced from git config, trusted on
//! review — not a cryptographic signature. Two mechanical corroborations do
//! the load-bearing work instead: this module's model/bot rejection (PRD §9.3,
//! verify class L006) and the blame-consistency check in [`crate::blame`]
//! (L009). A `signature` field is reserved on the acceptance schema, empty in
//! L0, so a cryptographic upgrade is additive rather than a migration.
//!
//! The identity is the **email**. §4.4 requires `accepted-by` to *resolve* to
//! a human identity; an address resolves, a display name decorates — and
//! comparing addresses rather than names also makes L009 robust against the
//! whitespace and punctuation drift real `user.name` values carry.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Local parts that name a machine rather than an accountable actor.
const RESERVED_LOCAL_PARTS: &[&str] = &[
    "actions", "automation", "bot", "build", "cd", "ci", "dependabot", "do-not-reply",
    "github-actions", "gitlab-ci", "jenkins", "no-reply", "noreply", "renovate", "robot",
];

/// Tokens naming a model actor. Matched as whole tokens (trailing digits
/// stripped, so `gpt4` and `claude5` are caught), never as substrings.
const MODEL_TOKENS: &[&str] = &[
    "agent", "ai", "aider", "chatgpt", "claude", "codex", "copilot", "cursor", "devin", "gemini",
    "gpt", "llama", "llm", "mistral", "model",
];

/// Vendor no-reply addresses, matched exactly.
const VENDOR_NOREPLY: &[&str] = &[
    "noreply@anthropic.com",
    "noreply@github.com",
    "noreply@google.com",
    "noreply@openai.com",
];

/// A claimed actor identity, normalised to lowercase.
///
/// Normalisation happens at parse because the identity is hashed content on
/// an escaped version's `accepted_by`: `Emil@Delegate.dk` and
/// `emil@delegate.dk` must not produce two version hashes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identity(String);

impl Identity {
    /// The normalised address.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or_default()
    }

    fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or_default()
    }

    /// Why this identity cannot be an acceptor, when it cannot (PRD §9.3).
    ///
    /// The list is a floor, not a proof: it catches the identities a CI
    /// system or an agent harness produces by default, which is where the
    /// failure actually occurs. It cannot catch a model configured with a
    /// human-looking address, and the format spec says so.
    pub fn model_or_bot_reason(&self) -> Option<String> {
        if VENDOR_NOREPLY.contains(&self.0.as_str()) {
            return Some(format!("`{}` is a vendor no-reply address", self.0));
        }
        if self.0.contains("[bot]") {
            return Some(format!("`{}` carries the `[bot]` marker", self.0));
        }
        let domain = self.domain();
        if domain == "github-actions.com" || domain.starts_with("github-actions.") {
            return Some(format!("`{}` is a CI runner domain", domain));
        }
        let local = self.local_part();
        if RESERVED_LOCAL_PARTS.contains(&local) {
            return Some(format!("`{local}` names a machine, not an accountable actor"));
        }
        model_token(local).map(|t| format!("`{local}` carries the model-actor token `{t}`"))
    }

    /// Whether this identity may sign an acceptance.
    pub fn is_acceptable(&self) -> bool {
        self.model_or_bot_reason().is_none()
    }
}

/// The first model token in a local part, matched whole rather than as a
/// substring — `alain@` must not read as `ai`, `claudia@` must not read as
/// `claude`.
fn model_token(local: &str) -> Option<&'static str> {
    local
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(|t| t.trim_end_matches(|c: char| c.is_ascii_digit()))
        .find_map(|t| MODEL_TOKENS.iter().copied().find(|m| *m == t))
}

impl FromStr for Identity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.chars().any(char::is_whitespace) {
            return Err(format!("`{raw}` carries whitespace; an identity is a bare address"));
        }
        let (local, domain) = raw
            .split_once('@')
            .ok_or_else(|| format!("`{raw}` is not an address — an identity resolves by email"))?;
        if local.is_empty() || domain.is_empty() {
            return Err(format!("`{raw}` has an empty local part or domain"));
        }
        if domain.contains('@') {
            return Err(format!("`{raw}` carries more than one `@`"));
        }
        Ok(Self(raw.to_lowercase()))
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Identity {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

#[path = "identity_tests.rs"]
#[cfg(test)]
mod tests;
