//! `product csharp …` — the C# stack binding's consumer side (Gate 1, 2026-09-13).
//!
//! Reads the inventory artefact the .NET reader (`tools/csharp-inventory`)
//! emits and measures it. `inventory` checks the artefact's version and
//! schema; `reach` is the §12.1 reachability measure from a stated root
//! convention; `delta` is the act-indexed three-region report plus the two
//! ratios, which needs the binding's event model. Nothing here writes to
//! `.product/`, and no output lists symbols as candidate slices (R-D).

use std::path::{Path, PathBuf};

use clap::Subcommand;
use product_core::error::ProductError;
use product_core::pf::csharp_delta::{delta, render_delta, DeltaOptions};
use product_core::pf::csharp_inventory::{load_inventory, schema_findings, Inventory};
use product_core::pf::csharp_reach::{reach, render_reach, ReachOptions, Root};
use product_core::pf::eventmodel::load_event_model;
use serde_json::json;

use super::output::{CmdResult, Output};

#[derive(Subcommand)]
pub enum CsharpCommands {
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
        /// Follow implement/inherit edges when computing the ratios
        #[arg(long = "through-implementations")]
        through_implementations: bool,
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
        /// attribute:<T:…>, implements:<T:…>, declared:<T:…>
        #[arg(long, default_value = "entry-point")]
        roots: String,
        /// Follow implement/inherit edges from a reached type to its
        /// implementors (a DI container's resolution, approximated)
        #[arg(long = "through-implementations")]
        through_implementations: bool,
    },
}

pub(crate) fn handle_csharp(cmd: CsharpCommands) -> CmdResult {
    match cmd {
        CsharpCommands::Inventory { file } => inventory_cmd(&file),
        CsharpCommands::Reach { file, roots, through_implementations } => {
            reach_cmd(&file, &roots, through_implementations)
        }
        CsharpCommands::Delta { file, event_model, slice_attribute, realises_fact_attribute, through_implementations } => {
            delta_cmd(&file, &event_model, DeltaOptions { slice_attribute, realises_fact_attribute, through_implementations })
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

fn reach_cmd(file: &Path, roots: &str, through_implementations: bool) -> CmdResult {
    let inv = load(file)?;
    let mut parsed = Vec::new();
    for spec in roots.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        parsed.push(Root::parse(spec).ok_or_else(|| {
            ProductError::ConfigError(format!(
                "unknown root convention '{spec}' — expected entry-point, public, attribute:<T:…>, implements:<T:…> or declared:<T:…>"
            ))
        })?);
    }
    let opts = ReachOptions { roots: parsed, through_implementations };
    let report = reach(&inv, &opts);
    let text = render_reach(&report, &opts);
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
