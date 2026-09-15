//! `product csharp …` — the C# stack binding's consumer side (Gate 1a, 2026-09-13).
//!
//! Reads the inventory artefact the .NET reader (`tools/csharp-inventory`)
//! emits and measures it. `inventory` checks the artefact's version and
//! schema; `reach` is the reachability measure from a stated root convention,
//! resolved through the container's registrations with unresolved edges
//! reported as their own category (CG-R-60 … 62); `delta` is the act-indexed
//! three-region report plus the ratios, which needs the binding's event model. Nothing here writes to
//! `.product/`, and no output lists symbols as candidate slices (R-D).

use std::path::{Path, PathBuf};

use clap::Subcommand;
use product_core::error::ProductError;
use product_core::pf::csharp_candidates_render::render_candidates;
use product_core::pf::csharp_candidates_report::{candidates, EntryPointTruth};
use product_core::pf::csharp_delta::{delta, render_delta, DeltaOptions};
use product_core::pf::csharp_inventory::{load_inventory, schema_findings, Inventory};
use product_core::pf::csharp_reach::{reach, render_reach, ReachOptions, Root};
use product_core::pf::csharp_regions::{regions, Ratification};
use product_core::pf::csharp_regions_render::render_regions;
use product_core::pf::eventmodel::load_event_model;
use serde_json::json;

use super::output::{CmdResult, Output};

#[derive(Subcommand)]
pub enum CsharpCommands {
    /// Entry-point candidates (Gate 1b): one per external integration point —
    /// controller action, Razor page handler, ViewComponent, FastEndpoints
    /// endpoint, hosted service — with observed positions, the path, and
    /// unfilled ground slots; a measurement vocabulary, transport-derived
    Candidates {
        /// Path to the inventory JSON
        file: PathBuf,
        /// A hand enumeration of the solution's integration points (YAML:
        /// entry_points[{kind,type,member,method}], not_visible[…]) for recall
        #[arg(long = "ground-truth")]
        ground_truth: Option<PathBuf>,
    },
    /// The act-indexed delta against an event model: declared, declarable,
    /// unstructured or unrealised per act, plus the two undeclared ratios
    Delta {
        /// Path to the inventory JSON
        file: PathBuf,
        /// Path to the binding's event model (act + fact vocabulary, YAML)
        #[arg(long = "event-model")]
        event_model: PathBuf,
        /// Type id of the slice attribute
        #[arg(long = "slice-attribute", default_value = "T:Product.Binding.SliceAttribute")]
        slice_attribute: String,
        /// Type id of the fact attribute
        #[arg(long = "realises-fact-attribute", default_value = "T:Product.Binding.RealisesFactAttribute")]
        realises_fact_attribute: String,
    },
    /// Check an inventory artefact: known version, conforms to the vendored
    /// schema, and a summary of what it carries
    Inventory {
        /// Path to the inventory JSON the .NET reader emitted
        file: PathBuf,
    },
    /// Reachability from a stated root convention (§12.1): how much of the
    /// solution is reached from entry points, the public surface, or symbols
    /// carrying a named attribute or implementing a named interface
    Reach {
        /// Path to the inventory JSON
        file: PathBuf,
        /// Root conventions, comma-separated: entry-point, public,
        /// attribute:<T:…>, implements:<T:…>, declared:<T:…>, member:<M:…>
        #[arg(long, default_value = "entry-point")]
        roots: String,
        /// Report a symbol set's disposition on its own, e.g.
        /// implements:T:MediatR.IRequestHandler`2 (repeatable)
        #[arg(long = "track")]
        track: Vec<String>,
        /// A hand-enumerated ground truth (YAML: project, edges[{from,target}])
        /// for reader recall and walk recall/precision (CG-R-71)
        #[arg(long = "ground-truth")]
        ground_truth: Option<PathBuf>,
    },
    /// The three-region delta over a ratified candidate set (Gate 1b): declared,
    /// declarable, unstructured or no-facts-under-proxy per entry point, the
    /// reachable/unresolved/isolated split over undeclared types with its
    /// error bound, and §12.1 read against it — transport-derived, graded as
    /// the worksheet is
    Regions {
        /// Path to the inventory JSON
        file: PathBuf,
        /// The filled ratification worksheet (YAML)
        #[arg(long)]
        ratification: PathBuf,
    },
}

pub(crate) fn handle_csharp(cmd: CsharpCommands) -> CmdResult {
    match cmd {
        CsharpCommands::Candidates { file, ground_truth } => candidates_cmd(&file, ground_truth.as_deref()),
        CsharpCommands::Inventory { file } => inventory_cmd(&file),
        CsharpCommands::Reach { file, roots, track, ground_truth } => reach_cmd(&file, &roots, &track, ground_truth.as_deref()),
        CsharpCommands::Regions { file, ratification } => regions_cmd(&file, &ratification),
        CsharpCommands::Delta { file, event_model, slice_attribute, realises_fact_attribute } => {
            delta_cmd(&file, &event_model, DeltaOptions { slice_attribute, realises_fact_attribute })
        }
    }
}

fn read(path: &Path) -> Result<String, ProductError> {
    std::fs::read_to_string(path)
        .map_err(|e| ProductError::IoError(format!("cannot read {}: {e}", path.display())))
}

fn load(path: &Path) -> Result<Inventory, ProductError> {
    load_inventory(&read(path)?)
}

fn inventory_cmd(file: &Path) -> CmdResult {
    let text = read(file)?;
    let inv = load_inventory(&text)?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ProductError::ConfigError(format!("inventory: {e}")))?;
    let findings = schema_findings(&value)?;
    if !findings.is_empty() {
        return Err(ProductError::ConfigError(format!(
            "inventory does not conform to schema version {}:\n  {}",
            inv.inventory_version,
            findings.join("\n  ")
        )));
    }
    let text = format!(
        "inventory version {} — {} ({}), produced by {} {} at {}\n{} project(s), {} type(s), {} member(s), {} reference(s), {} diagnostic(s)\n{}",
        inv.inventory_version,
        inv.solution.path,
        inv.solution.git_head.as_deref().unwrap_or("no git head"),
        inv.produced_by.tool,
        inv.produced_by.tool_version,
        inv.produced_at,
        inv.projects.len(),
        inv.types.len(),
        inv.members.len(),
        inv.references.len(),
        inv.diagnostics.len(),
        inv.diagnostics.iter().map(|d| format!("  {}: {}\n", d.severity, d.message)).collect::<String>(),
    );
    let json = json!({
        "inventory_version": inv.inventory_version,
        "solution": inv.solution,
        "produced_by": inv.produced_by,
        "counts": { "projects": inv.projects.len(), "types": inv.types.len(), "members": inv.members.len(), "references": inv.references.len(), "diagnostics": inv.diagnostics.len() },
        "diagnostics": inv.diagnostics,
    });
    Ok(Output::both(text, json))
}

fn parse_roots(specs: &[&str]) -> Result<Vec<Root>, ProductError> {
    specs
        .iter()
        .map(|spec| {
            Root::parse(spec).ok_or_else(|| {
                ProductError::ConfigError(format!(
                    "unknown root convention '{spec}' — expected entry-point, public, attribute:<T:…>, implements:<T:…>, declared:<T:…> or member:<M:…>"
                ))
            })
        })
        .collect()
}

fn reach_cmd(file: &Path, roots: &str, track: &[String], ground_truth: Option<&Path>) -> CmdResult {
    let inv = load(file)?;
    let root_specs: Vec<&str> = roots.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    let track_specs: Vec<&str> = track.iter().map(String::as_str).collect();
    let ground_truth = match ground_truth {
        Some(p) => Some(serde_yaml::from_str(&read(p)?).map_err(|e| ProductError::ConfigError(format!("ground truth {}: {e}", p.display())))?),
        None => None,
    };
    let opts = ReachOptions { roots: parse_roots(&root_specs)?, track: parse_roots(&track_specs)?, ground_truth };
    let report = reach(&inv, &opts);
    let text = render_reach(&report);
    let json = serde_json::to_value(&report).map_err(|e| ProductError::Internal(e.to_string()))?;
    Ok(Output::both(text, json))
}

fn candidates_cmd(file: &Path, ground_truth: Option<&Path>) -> CmdResult {
    let inv = load(file)?;
    let truth: Option<EntryPointTruth> = match ground_truth {
        Some(p) => Some(serde_yaml::from_str(&read(p)?).map_err(|e| ProductError::ConfigError(format!("ground truth {}: {e}", p.display())))?),
        None => None,
    };
    let report = candidates(&inv, truth.as_ref());
    let text = render_candidates(&report);
    let json = serde_json::to_value(&report).map_err(|e| ProductError::Internal(e.to_string()))?;
    Ok(Output::both(text, json))
}

fn regions_cmd(file: &Path, ratification: &Path) -> CmdResult {
    let inv = load(file)?;
    let rat: Ratification = serde_yaml::from_str(&read(ratification)?).map_err(|e| ProductError::ConfigError(format!("ratification {}: {e}", ratification.display())))?;
    let report = regions(&inv, &rat);
    let text = render_regions(&report);
    let json = serde_json::to_value(&report).map_err(|e| ProductError::Internal(e.to_string()))?;
    Ok(Output::both(text, json))
}

fn delta_cmd(file: &Path, event_model: &Path, opts: DeltaOptions) -> CmdResult {
    let inv = load(file)?;
    let model = load_event_model(&read(event_model)?)?;
    let report = delta(&inv, &model, &opts);
    let text = render_delta(&report);
    let json = serde_json::to_value(&report).map_err(|e| ProductError::Internal(e.to_string()))?;
    Ok(Output::both(text, json))
}
