//! The per-run evidence file — derivation keyed by assertion id.
//!
//! The PRD's ground model has a **per-source evidence layer**: one graph per
//! extraction run, pinned to the corpus ref, holding what the run saw. G0
//! collapsed evidence onto the assertion node for expedience, which is why
//! Group B was unfindable — the facts that would have named it were never
//! written anywhere.
//!
//! This is that layer doing its job. The derivation predicates ride on the
//! assertion's own node (identity is content-addressed, so the id *is* the
//! join key), but the triples live under `runs/`, leaving the canonical graph
//! pure ratified assertion. Two consequences follow, both load-bearing:
//!
//! - **The already-ratified cohort becomes auditable.** The extractor is
//!   deterministic and the corpus ref is pinned, so re-running at the ref an
//!   assertion was extracted from regenerates its derivation exactly. The
//!   sidecar is written; not one byte of ratified content changes.
//! - **A sidecar entry is checkable, never assumed.** An entry either names an
//!   assertion the graph holds or it is orphaned, and [`orphans`] reports the
//!   count per run rather than trusting the join.

use serde::Serialize;

use super::triple::ProposedAssertion;
use super::vocab::{reg, PREFIXES};
use super::write::{shorten, WriteContext};

/// Where run evidence lives inside an instance.
pub const RUNS_DIR: &str = "runs";

/// The file one run's evidence lands in.
pub fn file_name(corpus: &str, git_ref: &str) -> String {
    format!("{RUNS_DIR}/{}.ttl", super::mint::local_name(&format!("{corpus}-at-{git_ref}")))
}

/// The Turtle document holding one run's derivation records.
///
/// `entries` is everything the run read — proposed *and* already ratified.
/// Evidence is about what the instrument saw, not about what is new, and it is
/// exactly that distinction that lets a re-run at an old ref populate the
/// cohort ratified before this layer existed.
pub fn run_file(entries: &[ProposedAssertion], ctx: &WriteContext) -> String {
    let base = &ctx.base_iri;
    let mut out = header(entries.len(), ctx);
    out.push_str(&prefixes(base));
    out.push_str(&format!(
        "\n{}\n    a            reg:ExtractionRun ;\n    reg:corpus   {} ;\n    reg:sourceRef {} ;\n    reg:asOf     {}^^xsd:dateTime .\n",
        shorten(&ctx.run_iri, base),
        quoted(&ctx.corpus),
        quoted(&ctx.git_ref),
        quoted(&ctx.as_of),
    ));
    let mut sorted: Vec<&ProposedAssertion> = entries.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    sorted.dedup_by(|a, b| a.id == b.id);
    for entry in sorted {
        out.push_str(&record(entry, base, &ctx.run_iri));
    }
    out
}

/// One assertion's derivation, as predicates on the assertion node.
fn record(a: &ProposedAssertion, base: &str, run: &str) -> String {
    // `reg:mappingRow` is not repeated here: the canonical file already
    // carries it, and duplicating a fact across layers is how the two drift.
    let mut fields: Vec<(String, String)> =
        vec![("reg:derivedByRule".into(), quoted(a.derivation.rule))];
    for op in a.derivation.stands_on {
        fields.push(("reg:standsOn".into(), format!("reg:{}", reg::operation_name(op))));
    }
    if let Some(kind) = a.derivation.member_kind {
        fields.push(("reg:memberKind".into(), format!("reg:{}", kind.as_str())));
    }
    if let Some(kind) = a.derivation.container_kind {
        fields.push(("reg:containerKind".into(), format!("reg:{}", kind.as_str())));
    }
    let mut paths = a.derivation.source_paths.clone();
    paths.sort();
    paths.dedup();
    for path in paths {
        fields.push(("reg:sourcePath".into(), quoted(&path)));
    }
    fields.push(("prov:wasGeneratedBy".into(), shorten(run, base)));
    let width = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let mut out = format!("\nreg:assertion-{}\n", a.id);
    for (i, (key, value)) in fields.iter().enumerate() {
        let terminator = if i + 1 == fields.len() { "." } else { ";" };
        out.push_str(&format!("    {key:<width$}  {value} {terminator}\n"));
    }
    out
}

/// What the file is, stated in the file, so a reader never mistakes evidence
/// for ratified content.
fn header(count: usize, ctx: &WriteContext) -> String {
    format!(
        "# Extraction-run evidence — {} derivation record(s).\n#\n\
         # NOT ratified content. These are facts about how the instrument read\n\
         # {} at {}: the rule, the operations it stood on, the source kinds it\n\
         # saw. They ride on each assertion's own node, because identity is\n\
         # content-addressed and the id is the join key.\n\
         # Recomputable by construction — re-running at this ref regenerates\n\
         # this file, which is why a cohort ratified before this layer existed\n\
         # can still be given its derivation without a rewrite.\n\n",
        count, ctx.corpus, ctx.git_ref
    )
}

/// An entry naming an assertion the graph does not hold.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Orphan {
    /// The run file the entry sits in.
    pub run: String,
    /// The assertion id with no counterpart.
    pub assertion: String,
}

/// Sidecar entries with no assertion behind them.
///
/// Identity is content-addressed, so this is a real check rather than a
/// formality: an entry either matches a ratified assertion or it names one
/// that was never ratified — a proposal rejected at review, or a run against
/// the wrong instance. Both are findings, and both are silent without this.
pub fn orphans(
    run: &str,
    entry_ids: &[String],
    graph_ids: &std::collections::BTreeSet<String>,
) -> Vec<Orphan> {
    let mut out: Vec<Orphan> = entry_ids
        .iter()
        .filter(|id| !graph_ids.contains(*id))
        .map(|id| Orphan { run: run.to_string(), assertion: id.clone() })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every orphan across an instance's run files, run by run.
///
/// The mechanical form of the join condition: the sidecar is checkable
/// against the graph rather than assumed to line up with it.
pub fn orphans_in(instance: &std::path::Path) -> Vec<Orphan> {
    let graph_ids = assertion_ids_under(&instance.join("graphs"));
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(instance.join(RUNS_DIR)) else { return out };
    let mut runs: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("ttl"))
        .collect();
    runs.sort();
    for path in runs {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        out.extend(orphans(&name, &entry_ids(&text), &graph_ids));
    }
    out
}

/// Assertion ids a graph directory holds, read off the file names, which
/// carry the content-derived identity.
fn assertion_ids_under(dir: &std::path::Path) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(id) = name.strip_prefix("assertion-").and_then(|n| n.strip_suffix(".ttl")) {
                out.insert(id.to_string());
            }
        }
    }
    out
}

/// Assertion ids a run file names, read back out of its own text.
pub fn entry_ids(ttl: &str) -> Vec<String> {
    let mut out: Vec<String> = ttl
        .lines()
        .filter_map(|l| l.trim().strip_prefix("reg:assertion-"))
        .map(|rest| rest.trim().trim_end_matches(['.', ';', ' ']).to_string())
        .filter(|id| !id.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Prefix declarations, the instance base first.
fn prefixes(base: &str) -> String {
    let width = PREFIXES.iter().map(|(n, _)| n.len()).chain([3]).max().unwrap_or(4) + 1;
    let mut out = format!("@prefix {:<width$} <{base}> .\n", "reg:");
    for (name, iri) in PREFIXES {
        out.push_str(&format!("@prefix {:<width$} <{iri}> .\n", format!("{name}:")));
    }
    out
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", super::triple::escape_literal(value))
}

#[cfg(test)]
#[path = "sidecar_tests.rs"]
mod tests;
