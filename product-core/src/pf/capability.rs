//! Worker capability catalog + role bindings (ported from decision-cli).
//!
//! A `Capability` is a catalogued worker/model — its `endpoint` selects the
//! runner (`claude` subprocess vs the `litellm` proxy), `id` is the routing tag.
//! A `RoleBinding` binds a role (e.g. `implementer`) to a default capability
//! plus an escalation ladder: triggers (low confidence, repeated failure, …)
//! bump the resolved capability up the tiers. The graph/catalog is the source
//! of truth; this mirrors decision-cli's `dec:Capability`/`dec:RoleBinding`.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::validate::Violation;

/// Known runner endpoints. `litellm` (and the `scaleway`/`anthropic` aliases)
/// all route through the LiteLLM proxy at `LITELLM_BASE_URL`, which holds the
/// provider keys and maps the capability tag to a provider model — so Scaleway
/// is reached via a proxy model group, not a direct API call here.
pub const ENDPOINTS: &[&str] = &["claude", "litellm", "worker", "scaleway", "anthropic"];

/// The escalation triggers a role binding may name (fixed vocabulary).
pub const TRIGGERS: &[&str] = &[
    "audit_fail",
    "confidence_below_0.7",
    "confidence_below_0.5",
    "prior_attempts_ge_3",
    "prior_attempts_ge_5",
    "stakes_foundational",
    "feedback_unimplementable_critical",
];

/// A catalogued worker capability — a model behind an endpoint/runner.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Routing tag (also the litellm `model_name`).
    pub id: String,
    /// Runner selector: `claude` (subprocess) | `litellm` (HTTP proxy).
    pub endpoint: String,
    /// The underlying provider model (informational for claude; the litellm
    /// group is keyed by `id`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model_identifier: String,
    /// Escalation-ladder tier (higher = stronger/costlier).
    #[serde(default)]
    pub tier: u8,
    #[serde(default = "active")]
    pub status: String,
    /// Optional invocation parameters merged verbatim into the model request
    /// body (e.g. `max_tokens`, `temperature`, `chat_template_kwargs:
    /// {enable_thinking: false}` for reasoning models served by vLLM). The
    /// catalog owns the binding, so the binding owns its sampling knobs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invocation: Option<serde_json::Value>,
    /// Optional first-party-worker response mode: `files` forces whole-file
    /// rewrites (no targeted edits) — the reliable setting for smaller local
    /// models that cannot reproduce exact unique find-spans; unset or `edits`
    /// keeps edits-preferred when current file content is shown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_mode: Option<String>,
}

/// The response modes a capability may declare.
pub const RESPONSE_MODES: &[&str] = &["files", "edits"];

fn active() -> String {
    "active".to_string()
}

/// One rung of a role's escalation ladder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscalationStep {
    pub capability: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
}

/// A role → capability binding with an escalation ladder.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role_id: String,
    pub default_capability: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub escalation_steps: Vec<EscalationStep>,
    #[serde(default = "yes")]
    pub active: bool,
}

fn yes() -> bool {
    true
}

/// Wrapper for `capabilities.yaml` (`{ capabilities: [...] }`).
#[derive(Debug, Deserialize)]
struct CapabilityFile {
    #[serde(default)]
    capabilities: Vec<Capability>,
}

/// Wrapper for `role-bindings.yaml` (`{ role_bindings: [...] }`).
#[derive(Debug, Deserialize)]
struct RoleBindingFile {
    #[serde(default)]
    role_bindings: Vec<RoleBinding>,
}

/// The worker catalog: capabilities + role bindings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Catalog {
    pub capabilities: Vec<Capability>,
    pub role_bindings: Vec<RoleBinding>,
}

impl Catalog {
    pub fn capabilities_from_yaml(text: &str) -> Result<Vec<Capability>> {
        serde_yaml::from_str::<CapabilityFile>(text)
            .map(|f| f.capabilities)
            .map_err(|e| ProductError::ConfigError(format!("invalid capabilities YAML: {}", e)))
    }

    pub fn role_bindings_from_yaml(text: &str) -> Result<Vec<RoleBinding>> {
        serde_yaml::from_str::<RoleBindingFile>(text)
            .map(|f| f.role_bindings)
            .map_err(|e| ProductError::ConfigError(format!("invalid role-bindings YAML: {}", e)))
    }

    fn capability(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }

    fn binding(&self, role_id: &str) -> Option<&RoleBinding> {
        self.role_bindings.iter().find(|b| b.role_id == role_id && b.active)
    }

    /// Resolve a role to a capability, applying the escalation ladder against
    /// the active `triggers`. The highest rung whose triggers fire wins; with no
    /// firing trigger the default capability is used.
    pub fn resolve(&self, role_id: &str, triggers: &[String]) -> Option<&Capability> {
        let binding = self.binding(role_id)?;
        let mut chosen = binding.default_capability.as_str();
        for step in &binding.escalation_steps {
            if step.triggers.iter().any(|t| triggers.contains(t)) {
                chosen = step.capability.as_str();
            }
        }
        self.capability(chosen)
    }

    /// A role's capability ladder, weakest first: the default capability followed
    /// by each escalation rung, resolved to capabilities (unknown ids skipped,
    /// duplicates removed). A fix loop climbs this rung by rung as rounds fail.
    pub fn ladder(&self, role_id: &str) -> Vec<Capability> {
        let Some(binding) = self.binding(role_id) else {
            return Vec::new();
        };
        let mut ids = vec![binding.default_capability.clone()];
        ids.extend(binding.escalation_steps.iter().map(|s| s.capability.clone()));
        let mut out: Vec<Capability> = Vec::new();
        for id in ids {
            if let Some(c) = self.capability(&id) {
                if !out.iter().any(|x| x.id == c.id) {
                    out.push(c.clone());
                }
            }
        }
        out
    }
}

/// Validate a catalog: every binding's default + step capabilities resolve, and
/// triggers come from the fixed vocabulary.
pub fn validate_catalog(catalog: &Catalog) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in &catalog.capabilities {
        if !ENDPOINTS.contains(&c.endpoint.as_str()) {
            out.push(v(&c.id, "endpoint",
                &format!("unknown endpoint '{}' (expected one of: {})", c.endpoint, ENDPOINTS.join(", "))));
        }
        if let Some(mode) = &c.response_mode {
            if !RESPONSE_MODES.contains(&mode.as_str()) {
                out.push(v(&c.id, "response_mode",
                    &format!("unknown response_mode '{}' (expected one of: {})", mode, RESPONSE_MODES.join(", "))));
            }
        }
    }
    let known: std::collections::BTreeSet<&str> = catalog.capabilities.iter().map(|c| c.id.as_str()).collect();
    for b in &catalog.role_bindings {
        if !known.contains(b.default_capability.as_str()) {
            out.push(v(&b.role_id, "default_capability",
                &format!("default_capability '{}' is not a catalogued capability", b.default_capability)));
        }
        for s in &b.escalation_steps {
            if !known.contains(s.capability.as_str()) {
                out.push(v(&b.role_id, "escalation_steps",
                    &format!("escalation capability '{}' is not catalogued", s.capability)));
            }
            for t in &s.triggers {
                if !TRIGGERS.contains(&t.as_str()) {
                    out.push(v(&b.role_id, "triggers", &format!("unknown trigger '{t}'")));
                }
            }
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
#[path = "capability_tests.rs"]
mod tests;
