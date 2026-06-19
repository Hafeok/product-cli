//! Onboarding guidance — tells a user where they are in the framework graph's
//! What → How → Delivery journey and the exact next command to run.
//!
//! The framework graph (bounded contexts, entities, events, commands, read
//! models, deciders, the How contract, slices, deliverables) has all its
//! machinery but no on-ramp: each command stands alone and nothing connects
//! them. `guide` is the spine — it probes the graph's state and returns the
//! current [`Stage`] plus the concrete next step(s), papering over the
//! authoring papercuts (relations required up front, `slice --anchor`).

use std::path::Path;

mod plan;
mod render;
#[cfg(test)]
mod tests;

pub use plan::guide;
pub use render::render_text;

/// A snapshot of the framework graph's state for one product, read from disk.
/// Pure data — [`guide`] turns it into [`Guidance`] without touching the disk,
/// so the decision logic is unit-testable and CLI/MCP share one probe.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct FrameworkState {
    pub product: String,
    /// Total domain (What) nodes captured.
    pub what_total: usize,
    pub contexts: usize,
    pub entities: usize,
    pub commands: usize,
    pub events: usize,
    pub read_models: usize,
    /// Blocking conformance violations in the What graph (0 = conformant).
    pub violations: usize,
    /// An example command id, for a concrete `slice --anchor` suggestion.
    pub first_command: Option<String>,
    /// A How contract (`how-contract.yaml`) is present and parses.
    pub has_how: bool,
    pub deciders: usize,
    pub projectors: usize,
    pub slices: usize,
    /// An example slice id, for a concrete `deliverable --slice` suggestion.
    pub first_slice: Option<String>,
    pub deliverables: usize,
    pub releases: usize,
}

impl FrameworkState {
    /// Read the framework-graph state for `product` rooted at `repo_root`
    /// (the directory containing `.product/`). Missing pieces read as zero, so
    /// a fresh repo yields an all-zero state rather than an error.
    pub fn probe(repo_root: &Path, product: &str) -> Self {
        let pdir = repo_root.join(".product");
        let session = crate::author::domain::session_dir(repo_root, product);
        let graph = crate::pf::session::DomainSession::load(&session)
            .ok()
            .map(|s| s.graph);
        let what = probe_what(graph.as_ref());
        let has_how = std::fs::read_to_string(pdir.join("how-contract.yaml"))
            .ok()
            .and_then(|t| crate::pf::how::HowContract::from_yaml(&t).ok())
            .is_some();

        FrameworkState {
            product: product.to_string(),
            what_total: what.total,
            contexts: what.contexts,
            entities: what.entities,
            commands: what.commands,
            events: what.events,
            read_models: what.read_models,
            violations: what.violations,
            first_command: what.first_command,
            has_how,
            deciders: count_yaml(&pdir.join("deciders")),
            projectors: count_yaml(&pdir.join("projectors")),
            slices: count_yaml(&pdir.join("slices")),
            first_slice: first_yaml_stem(&pdir.join("slices")),
            deliverables: count_yaml(&pdir.join("deliverables")),
            releases: count_yaml(&pdir.join("releases")),
        }
    }
}

/// The What-graph half of the probe, kept separate so [`FrameworkState::probe`]
/// stays small and the graph derivation is one place.
struct WhatProbe {
    total: usize,
    contexts: usize,
    entities: usize,
    commands: usize,
    events: usize,
    read_models: usize,
    violations: usize,
    first_command: Option<String>,
}

fn probe_what(graph: Option<&crate::pf::model::DomainGraph>) -> WhatProbe {
    let count = |kind: &str| {
        graph
            .map(|g| g.counts().iter().find(|(k, _)| *k == kind).map(|(_, c)| *c).unwrap_or(0))
            .unwrap_or(0)
    };
    WhatProbe {
        total: graph.map(|g| g.counts().iter().map(|(_, c)| *c).sum()).unwrap_or(0),
        contexts: count("BoundedContext"),
        entities: count("Entity"),
        commands: count("Command"),
        events: count("Event"),
        read_models: count("ReadModel"),
        violations: graph.map(|g| crate::pf::validate::validate_graph(g).len()).unwrap_or(0),
        first_command: graph.and_then(|g| g.commands.first().map(|c| c.id.clone())),
    }
}

/// The stem of the first `*.yaml` file directly under `dir` (sorted for
/// determinism), or `None`. Used to name a concrete slice in guidance.
fn first_yaml_stem(dir: &Path) -> Option<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "yaml").unwrap_or(false))
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    names.into_iter().next()
}

/// Where the user is in the framework journey. Each stage names exactly one
/// next move, so guidance is never ambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    /// No What captured yet — the journey hasn't started.
    CaptureWhat,
    /// What exists but has blocking conformance violations.
    FixWhat,
    /// What is conformant; no How contract yet.
    AuthorHow,
    /// How exists; no delivery slice carved.
    CarveSlice,
    /// A slice exists; no deliverable wraps it.
    WrapDeliverable,
    /// A deliverable exists — make behaviour executable and build.
    BuildIt,
}

/// One recommended next action: the command to run and why.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NextStep {
    pub command: String,
    pub why: String,
}

impl NextStep {
    fn new(command: impl Into<String>, why: impl Into<String>) -> Self {
        NextStep { command: command.into(), why: why.into() }
    }
}

/// The full guidance result: where you are, what it means, and what to do.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Guidance {
    pub stage: Stage,
    pub headline: String,
    /// A one-line plain-language reminder of the concept this stage is about.
    pub concept: String,
    pub next_steps: Vec<NextStep>,
    /// The journey checklist: (label, done) in order.
    pub progress: Vec<(String, bool)>,
}

/// Count `*.yaml` files directly under `dir` (0 if the directory is absent).
fn count_yaml(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "yaml").unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}
