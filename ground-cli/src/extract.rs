//! The `ground extract` command — read, plan, gate, write, report.
//!
//! The order is load-bearing: nothing reaches the instance's working tree
//! until the plan has passed the instance's own shapes, so a reviewer never
//! reads per-triple diffs of content the registry's CI would reject.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ddd_lsp::host::Host;
use product_core::ground::{
    apply_extraction, plan_extraction, render_extraction, ExtractionParams, ExtractionPlan,
};

use crate::read::{self, ReadOutcome, ReadRequest};

/// Everything the command needs.
pub struct ExtractArgs {
    pub corpus: String,
    pub git_ref: String,
    pub repo_root: PathBuf,
    /// Directories under the root to read, or the root itself.
    pub sources: Vec<PathBuf>,
    /// Where the language host is rooted (a solution directory, often not
    /// the corpus root).
    pub workspace: PathBuf,
    pub host_command: Vec<String>,
    pub instance: PathBuf,
    pub graph_dir: String,
    pub as_of: String,
    pub instrument: String,
    /// Plan only: gate, report, write nothing.
    pub dry_run: bool,
}

/// What the run produced.
pub struct ExtractOutcome {
    pub plan: ExtractionPlan,
    pub read: ReadOutcome,
    pub written: Vec<String>,
    /// The run planned only, on request.
    pub dry_run: bool,
}

/// Run one extraction.
pub fn extract(args: &ExtractArgs) -> Result<ExtractOutcome, String> {
    let adapter = ddd_lsp::adapter::for_language("csharp").ok_or("no csharp adapter")?;
    let files = source_files(&args.sources, adapter.extensions);
    let probe_file = files.first().cloned().ok_or("no source files under the given sources")?;
    let mut host = Host::new(adapter, args.host_command.clone(), args.workspace.clone());
    let outcome = read::read(
        &mut host,
        &ReadRequest {
            corpus: args.corpus.clone(),
            git_ref: args.git_ref.clone(),
            repo_root: args.repo_root.clone(),
            files,
            probe_file,
            instrument: args.instrument.clone(),
        },
    )?;
    let base_iri = instance_base(&args.instance)?;
    let shapes = shapes_of(&args.instance)?;
    let ratified = ratified_ids(&args.instance.join(&args.graph_dir));
    let params = ExtractionParams {
        base_iri,
        as_of: args.as_of.clone(),
        graph_dir: args.graph_dir.clone(),
    };
    let plan =
        plan_extraction(&outcome.facts, &params, &shapes, &ratified).map_err(|e| e.to_string())?;
    let written = if args.dry_run {
        Vec::new()
    } else {
        apply_extraction(&plan, &args.instance).map_err(|e| e.to_string())?.written
    };
    Ok(ExtractOutcome { plan, read: outcome, written, dry_run: args.dry_run })
}

/// The run, as text for a reader or JSON for a query.
pub fn present(outcome: &ExtractOutcome, json: bool) -> String {
    if !json {
        return render(outcome);
    }
    let value = serde_json::json!({
        "plan": outcome.plan,
        "probe": outcome.read.probe,
        "read": {
            "declarations": outcome.read.facts.declarations.len(),
            "hierarchy_edges": outcome.read.facts.hierarchy.len(),
            "reference_sites": outcome.read.facts.references.len(),
            "unread_files": outcome.read.facts.unread_files,
            "generic_declarations": outcome.read.generic_declarations,
            "nested_declarations": outcome.read.nested_declarations,
            "field_members": outcome.read.field_members,
            "partial_declarations": outcome.read.partial_declarations,
            "name_collisions": outcome.read.name_collisions,
            "unconsumed_operations": outcome.read.unconsumed,
        },
        "written": outcome.written,
        "dry_run": outcome.dry_run,
    });
    format!("{}\n", serde_json::to_string_pretty(&value).unwrap_or_default())
}

/// The report a reviewer reads before the diff.
pub fn render(outcome: &ExtractOutcome) -> String {
    let mut out = outcome.read.probe.report();
    out.push('\n');
    out.push_str(&render_extraction(&outcome.plan));
    out.push_str(&read_notes(outcome));
    out.push_str(&outcome_line(outcome));
    out
}

/// Why nothing was written, when nothing was. Three different states, and
/// collapsing them into one line is the mistake this whole shape avoids
/// elsewhere: a dry run, a run with nothing new to propose, and a run that
/// wrote files are not the same report.
fn outcome_line(outcome: &ExtractOutcome) -> String {
    if outcome.dry_run {
        return "\nnothing written — planned only, on request\n".to_string();
    }
    if !outcome.written.is_empty() {
        return format!("\n{} file(s) written\n", outcome.written.len());
    }
    if outcome.plan.files.is_empty() {
        return format!(
            "\nnothing written — every triple this run read is already ratified \
             ({} assertion(s) in the graph); identity is content-derived, so a re-read \
             at the same ref is the same assertion\n",
            outcome.plan.already_ratified
        );
    }
    "\nnothing written — every planned file was already on disk, byte-identical\n".to_string()
}

fn read_notes(outcome: &ExtractOutcome) -> String {
    let r = &outcome.read;
    let mut out = String::from("\nthe read itself:\n");
    out.push_str(&format!("  {} declaration(s)\n", r.facts.declarations.len()));
    out.push_str(&format!("  {} hierarchy edge(s)\n", r.facts.hierarchy.len()));
    out.push_str(&format!("  {} reference site(s)\n", r.facts.references.len()));
    out.push_str(&format!(
        "  {} file(s) opened with no declaration read\n",
        r.facts.unread_files.len()
    ));
    out.push_str(&format!("  {} generic declaration(s)\n", r.generic_declarations));
    out.push_str(&format!(
        "  {} nested declaration(s) — their module is the enclosing namespace, not the outer type\n",
        r.nested_declarations
    ));
    out.push_str(&format!(
        "  {} member(s) read as fields rather than properties — the declaration surface \
         carries no visibility, so implementation is indistinguishable from domain structure here\n",
        r.field_members
    ));
    out.push_str(&format!(
        "  {} type(s) declared across several files (C# `partial` — one type, not two){}\n",
        r.partial_declarations.len(),
        list(&r.partial_declarations)
    ));
    out.push_str(&format!(
        "  {} simple-name collision(s) across modules{}\n",
        r.name_collisions.len(),
        if r.name_collisions.is_empty() {
            " — name-based range resolution is safe on this corpus".to_string()
        } else {
            format!(" — name-based range resolution would conflate: {}", r.name_collisions.join(", "))
        }
    ));
    out.push_str(&format!(
        "  operations that answered but no row consumes: {}\n",
        if r.unconsumed.is_empty() { "none".to_string() } else { r.unconsumed.join(", ") }
    ));
    out
}

/// A bounded, comma-joined list, or nothing when there is nothing to list.
fn list(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    format!(" — {}", items.join(", "))
}

/// Source files under the given roots, sorted so a run is reproducible.
pub fn source_files(sources: &[PathBuf], extensions: &[&str]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack: Vec<PathBuf> = sources.to_vec();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(name.as_str(), ".git" | "bin" | "obj" | "node_modules" | "target") {
                    stack.push(path);
                }
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| extensions.contains(&e.to_ascii_lowercase().as_str()))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The instance's base IRI, read from its own birth provenance.
fn instance_base(instance: &Path) -> Result<String, String> {
    let path = instance.join("GENERATION.ttl");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("@prefix reg:") else { continue };
        let inner = rest.trim().trim_start_matches('<');
        if let Some((iri, _)) = inner.split_once('>') {
            return Ok(iri.to_string());
        }
    }
    Err(format!("{} declares no reg: prefix", path.display()))
}

/// Every shape the instance ships, concatenated.
fn shapes_of(instance: &Path) -> Result<String, String> {
    let dir = instance.join("shapes");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    paths.sort();
    let mut out = String::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        out.push_str(&text);
        out.push('\n');
    }
    Ok(out)
}

/// Assertion ids the graph directory already holds — read off the file names,
/// which carry the content-derived identity.
pub fn ratified_ids(graph_dir: &Path) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(graph_dir) else { return BTreeSet::new() };
    entries
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .filter_map(|name| name.strip_prefix("assertion-")?.strip_suffix(".ttl").map(str::to_string))
        .collect()
}

#[cfg(test)]
#[path = "extract_tests.rs"]
mod tests;
