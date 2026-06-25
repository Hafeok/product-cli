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
    /// §4.5 — style values the screen carries; each must be a design-system
    /// token reference, never a literal (tokens-not-literals).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub styles: Vec<String>,
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
    /// §3.2.5 — the system this flow belongs to (a flow belongs to exactly one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
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

/// §3.2.5 — a first-class system: the named thing a page graph and flows belong
/// to. It owns a `root` and its flows but shares the domain model, so a What may
/// declare several systems over one domain (a customer app + an admin website),
/// each a distinct surface with its own root, flows, and target contexts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct System {
    pub id: String,
    pub label: String,
    /// What sort of thing it is (application, website, service, cli, …).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    /// One sentence of what it is for, in the ubiquitous language.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub purpose: String,
    /// §3.2.2 — the platform dimension of context of use (iOS, Android, web).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_platforms: Vec<String>,
    /// §3.2.2 — the gating interaction classes it targets (gui, tui, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_classes: Vec<String>,
    /// §3.2.4 — the ApplicationRoot its page graph roots at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
}

/// §3.2.0 — a Trigger: what initiates a command. Its `source` is exactly one of
/// `user` (a UI step), `external` (an API caller), or `automated` (a process
/// that reads a View and issues a command with no human in the loop). The
/// Automation and Translation patterns are an automated trigger that `watches` a
/// View; a Translation additionally reads from one source system.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Trigger {
    pub id: String,
    pub label: String,
    /// user | external | automated.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    /// The command this trigger issues.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub issues: String,
    /// §3.2.0 — the View an automated trigger watches (Automation/Translation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watches: Option<String>,
    /// §3.2.0 — the source system a Translation trigger reads events from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translates_from: Option<String>,
}

/// §4.5 — a declared, deliberate coverage gap: an AIO that cannot honestly be
/// reified in an interaction class (e.g. a `display-collection` of images in a
/// TUI). A *recorded* gap that carries a rationale, never a silent omission —
/// the same honesty as tagging a WCAG criterion manual or naming the Polanyi
/// floor: the framework records the boundary instead of papering over it.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UnreifiableRule {
    pub id: String,
    pub aio: String,
    /// The interaction class (gui / tui) the AIO is unreifiable in.
    pub class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
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

/// §4.5 — the design system: the closed CIO catalog + token surface a screen
/// composes from (`cios`, `tokens`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DesignSystem {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cios: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<String>,
}

/// §4.5 — a Concrete Interaction Object: an on-system component an AIO reifies to.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Cio {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// §4.5 — a design token (colour, spacing, typography, …).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Token {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// §4.5 — a reify(AIO, context) → CIO rule with rationale (the UX reasoning).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ReificationRule {
    pub id: String,
    pub aio: String,
    pub context: String,
    pub cio: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}
