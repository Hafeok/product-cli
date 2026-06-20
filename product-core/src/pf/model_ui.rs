//! The §3.2.1–§4.6 UI-layer node model (UI steps, AIOs, page graph,
//! accessibility, content) — split from model.rs for the 400-line gate.

use serde::{Deserialize, Serialize};

/// §3.2.1 — one (projection, display-AIO) the step surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Surface {
    pub projection: String,
    pub aio: String,
}

/// §3.2.1 — one (command, action-AIO) the step offers.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Offer {
    pub command: String,
    pub aio: String,
}

/// §3.2.1 — what a surfaced projection's state *means to the user*, or a
/// `waiver` (with reason) for an ignorable state. Exactly one is set.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct StateMeaning {
    pub projection: String,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meaning: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waiver: Option<String>,
}

/// §3.2.1 — a UI step (the What of a screen). Supersedes the free-text
/// `triggers`/`displays` (kept as a deprecated alias) with typed edges: the
/// projections it `surfaces` and commands it `offers`, each through an AIO, and
/// the steps it `transitions_to`. `intent` is the one permitted free-text
/// residue.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct WireframeStep {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surfaces: Vec<Surface>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offers: Vec<Offer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions_to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_meanings: Vec<StateMeaning>,
    /// §3.2.3 — screen-specific WCAG criteria added on top of the AIO-inherited
    /// union.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must_satisfy: Vec<String>,
    /// §3.2.1 — content references (key + role); the words are resolved by the
    /// How against a content store (§4.6), never stored as literals here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_refs: Vec<ContentRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub displays: Option<String>,
}

/// §3.2 — an ordered behaviour assembling steps into a timeline. §3.2.4 — a
/// named connected subgraph of the page graph with a declared `entry_page`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Flow {
    pub id: String,
    pub label: String,
    pub steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_page: Option<String>,
}

/// §3.2.4 — the distinguished node of the page graph; its `navigates_from_root`
/// out-edges are the global destinations (a page is "top-level" iff linked here).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ApplicationRoot {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub navigates_from_root: Vec<String>,
}

/// §3.2.2 — an Abstract Interaction Object: a named, modality-independent kind
/// of interaction a UI step is typed against. The closed core lives in
/// `ids::CORE_AIOS`; this node registers an adopter's additional AIOs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Aio {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub means: Option<String>,
    /// §3.2.3 — WCAG criteria this AIO carries; inherited by steps that use it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub must_satisfy: Vec<String>,
}

/// §3.2.2 — a declared context of use (form factor, modality, …) — a What-side
/// fact carrying no realisation; the parameter reification rules are written
/// against.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContextOfUse {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// §3.2.3 — an ingested WCAG 2.2 success criterion (`verification`:
/// machine/assisted/manual; `level`: A/AA/AAA; `satisfied`: the machine gate).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct WcagCriterion {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
    #[serde(default)]
    pub satisfied: bool,
}

/// §3.2.1 — a content reference: standing authored words a step carries, by key
/// with a declared role (heading/body/empty-message/error-message/help/legal) —
/// never a literal string.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContentRef {
    pub key: String,
    pub role: String,
}

/// §4.6 — one resolution in a content store: a (key, locale) → string mapping.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Resolution {
    pub key: String,
    pub locale: String,
    pub value: String,
}

/// §4.6 — the swappable provider of words. Declares the `locales` it covers and
/// the `resolutions` for (key, locale) pairs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContentStore {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locales: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolutions: Vec<Resolution>,
}

/// §3.2.3 — a dated, attributed record that a non-machine criterion was met.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Attestation {
    pub id: String,
    pub step: String,
    pub criterion: String,
    pub date: String,
    pub by: String,
}
