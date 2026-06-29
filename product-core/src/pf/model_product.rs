//! The §3.0–§3.6 product-boundary node model (product, journey, quality demand).
//!
//! Split from model.rs for the 400-line gate. These sit *above* the per-system
//! What: a product owns domains + systems (§3.0), a journey composes single-system
//! flows across Translation crossings (§3.0.1), and a quality demand bounds an
//! element with a checkable non-functional requirement (§3.6).

use serde::{Deserialize, Serialize};

/// §3.0 — the product: the root of the What. It owns one or more domains (the
/// shared meaning and rules) and one or more systems (the surfaces); a system
/// *references* whole domains rather than owning them, so domain concepts are
/// shared one level up. It carries little of its own — a name, a purpose, and
/// the What-version it is at (§7.3).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Product {
    pub id: String,
    pub label: String,
    /// One sentence of what the product is, in the ubiquitous language.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    /// §3.0 — the domains (bounded contexts) this product owns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owns_domain: Vec<String>,
    /// §3.0 — the systems this product owns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owns_system: Vec<String>,
    /// §7.3 — the product's What-version (the meaning's semantic version).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// §3.0.1 — a journey: a product-level, derived composition of single-system
/// flows linked at crossings, where every crossing is a Translation (§3.2.0). It
/// references flows and Translations that already exist and owns nothing — it
/// adds end-to-end cross-system visibility, not behaviour.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct Journey {
    pub id: String,
    pub label: String,
    /// §3.0.1 — the product this journey belongs to.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub product: String,
    /// §3.0.1 — the single-system flows it composes (in path order).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composes_flow: Vec<String>,
    /// §3.0.1 — the Translation triggers (§3.2.0) each crossing goes through.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub crosses_via: Vec<String>,
}

/// §3.6 — a quality demand (a non-functional requirement) made checkable. It is
/// exactly one of two kinds, distinguished by *how it is checked*: a
/// `runtime-bound` measured continuously against telemetry (the data-conformance
/// pattern over operational data), or an `architectural` constraint bound to the
/// How (§4) and checked at build time. It is located on the element it scopes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, schemars::JsonSchema)]
pub struct QualityDemand {
    pub id: String,
    pub label: String,
    /// `runtime-bound` | `architectural`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    /// The declared bound or constraint (e.g. "p99 latency ≤ 200ms").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bound: String,
    /// The element it bounds — a system, flow, ui-step, or Decider id.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scopes: String,
    /// §3.6 — a runtime bound's telemetry source it is measured against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured_by: Option<String>,
    /// §3.6 — an architectural constraint's How-side contract it binds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constrains: Option<String>,
}
