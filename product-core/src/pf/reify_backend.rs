//! Backend registry — the built-in language targets behind one trait.
//!
//! A [`ReifyBackend`] renders the deterministic oracle into one
//! ecosystem's verification shell. Built-ins live here (C# full/oracle,
//! Kotlin oracle-only); external backends never touch this trait — they
//! are processes consuming the [`super::reify_manifest`] JSON on stdin
//! and answering a file plan, wired via `product reify plugin`.

use crate::error::{ProductError, Result};

use super::decider::Decider;
use super::model::DomainGraph;
use super::projector::Projector;
use super::reify::{plan_csharp, ReifyOptions, ReifyPlan};
use super::reify_kotlin::plan_kotlin;

/// One built-in language backend.
pub trait ReifyBackend: Sync {
    /// The id used by `product reify emit --lang <id>` and the MCP tool.
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// True when the backend only supports the oracle-only tier.
    fn oracle_only_forced(&self) -> bool;
    fn plan(
        &self,
        graph: &DomainGraph,
        deciders: &[Decider],
        projectors: &[Projector],
        opts: &ReifyOptions,
    ) -> Result<ReifyPlan>;
}

struct Csharp;
struct Kotlin;

impl ReifyBackend for Csharp {
    fn id(&self) -> &'static str {
        "csharp"
    }
    fn description(&self) -> &'static str {
        "C# / .NET 8 — full mode (typed records, Decider/Projector frames, xUnit facts) or --oracle-only (adapter seam)"
    }
    fn oracle_only_forced(&self) -> bool {
        false
    }
    fn plan(&self, graph: &DomainGraph, deciders: &[Decider], projectors: &[Projector], opts: &ReifyOptions) -> Result<ReifyPlan> {
        plan_csharp(graph, deciders, projectors, opts)
    }
}

impl ReifyBackend for Kotlin {
    fn id(&self) -> &'static str {
        "kotlin"
    }
    fn description(&self) -> &'static str {
        "Kotlin / JVM (Gradle) — oracle-only: wire seam, kotlin.test facts, conform runner; the realiser owns the domain design"
    }
    fn oracle_only_forced(&self) -> bool {
        true
    }
    fn plan(&self, graph: &DomainGraph, deciders: &[Decider], projectors: &[Projector], opts: &ReifyOptions) -> Result<ReifyPlan> {
        plan_kotlin(graph, deciders, projectors, opts)
    }
}

static BACKENDS: [&dyn ReifyBackend; 2] = [&Csharp, &Kotlin];

/// Every built-in backend, in registration order.
pub fn backends() -> &'static [&'static dyn ReifyBackend] {
    &BACKENDS
}

/// Parse an external backend's answer — `{"files": [{path, content,
/// overwrite?}]}` — into a [`ReifyPlan`], appending the provenance manifest
/// so `product reify check` (and stale-file cleanup) work on plugin trees
/// exactly as on built-in ones. Paths must be relative and stay inside the
/// output tree.
pub fn external_plan(
    stdout_json: &str,
    graph: &DomainGraph,
    deciders: &[Decider],
    projectors: &[Projector],
    opts: &ReifyOptions,
) -> Result<ReifyPlan> {
    use super::reify::{aggregate_names, input_hash, provenance_json, GenFile};
    let v: serde_json::Value = serde_json::from_str(stdout_json).map_err(|e| {
        ProductError::ConfigError(format!("plugin output is not valid JSON: {e}"))
    })?;
    let entries = v
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| ProductError::ConfigError("plugin output carries no `files` array".to_string()))?;
    let mut files = Vec::new();
    for e in entries {
        let path = e
            .get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| ProductError::ConfigError("plugin file entry missing `path`".to_string()))?;
        if path.starts_with('/') || path.split('/').any(|seg| seg == "..") {
            return Err(ProductError::ConfigError(format!(
                "plugin file path '{path}' must be relative and stay inside the output tree"
            )));
        }
        files.push(GenFile {
            path: path.to_string(),
            content: e.get("content").and_then(|c| c.as_str()).unwrap_or_default().to_string(),
            overwrite: e.get("overwrite").and_then(|o| o.as_bool()).unwrap_or(true),
        });
    }
    let graph_hash = input_hash(graph, &opts.product, deciders, projectors)?;
    let mut sorted: Vec<&Decider> = deciders.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let aggregates = aggregate_names(&sorted)?;
    files.push(GenFile {
        path: "provenance.g.json".to_string(),
        content: provenance_json(opts, &graph_hash, &files),
        overwrite: true,
    });
    Ok(ReifyPlan { files, graph_hash, aggregates })
}

/// Resolve a built-in backend by id.
pub fn backend(id: &str) -> Result<&'static dyn ReifyBackend> {
    BACKENDS
        .iter()
        .find(|b| b.id() == id)
        .copied()
        .ok_or_else(|| {
            let known: Vec<&str> = BACKENDS.iter().map(|b| b.id()).collect();
            ProductError::ConfigError(format!(
                "unknown reify backend '{id}' — built-ins: {} (external backends run via `product reify plugin`)",
                known.join(", ")
            ))
        })
}
