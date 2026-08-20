//! Conformance reader for a generated registry instance.
//!
//! Runs the two rules an instance's own CI runs — the file rule (every
//! `graphs/**/*.ttl` carries exactly one assertion or exactly one decision) and
//! the SHACL shapes in `shapes/` — over a tree on disk.
//!
//! It reads a **defined subset** of SHACL, compiled to SPARQL, so the shapes
//! files stay the single statement of the constraints: nothing is restated in
//! Rust. Anything outside that subset is reported as *unevaluable* and **fails**
//! the check. A shape the reader cannot read never passes silently, and an
//! unevaluable shape is reported distinctly from data violating a shape.
//!
//! This is a second reader of rules `pyshacl` also reads in the instance's CI.
//! The divergence is a standing finding, measured by a fixture that runs both
//! when pySHACL is present rather than assumed small.

pub(crate) mod eval;
pub(crate) mod file_rule;
pub(crate) mod shapes;

use std::path::Path;

use crate::error::{ProductError, Result};

/// What kind of finding this is — the three are reported separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    /// A file broke the one-assertion-or-one-decision rule.
    FileRule,
    /// Data violated a shape the reader evaluated.
    ShapeViolation,
    /// A shape the reader could not read. The check fails closed.
    Unevaluable,
}

/// One finding.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Finding {
    /// Which kind of finding.
    pub kind: FindingKind,
    /// The file it was found in, where one applies.
    pub file: Option<String>,
    /// The focus node, shape, or construct at fault.
    pub focus: String,
    /// What is wrong.
    pub message: String,
}

/// The result of a check.
#[derive(Debug, Default, serde::Serialize)]
pub struct CheckReport {
    /// Data files read.
    pub files_checked: usize,
    /// Constraints evaluated.
    pub constraints_evaluated: usize,
    /// Everything found.
    pub findings: Vec<Finding>,
}

impl CheckReport {
    /// True when nothing was found — no violation, no unreadable shape.
    pub fn conforms(&self) -> bool {
        self.findings.is_empty()
    }

    /// Findings of one kind.
    pub fn of_kind(&self, kind: FindingKind) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.kind == kind).collect()
    }
}

/// Read an instance at `dir` against its own rules.
///
/// The **file rule** applies to `graphs/` alone: only a graph file must carry
/// exactly one assertion or one decision. The **shapes** run over the join —
/// `graphs/`, `runs/`, `vocabulary/` — because that is what a consumer reads,
/// and because the derivation half of an assertion's shape lives in the run
/// evidence. Validating the graph layer alone would pass shapes the projection
/// then fails.
pub fn check_instance(dir: &Path) -> Result<CheckReport> {
    let graph_files = ttl_files(&dir.join("graphs"))?;
    let run_files = ttl_files(&dir.join("runs"))?;
    let vocabulary = ttl_files(&dir.join("vocabulary"))?;
    let shape_files = ttl_files(&dir.join("shapes"))?;
    let base = base_iri(dir)?;

    let mut report = CheckReport { files_checked: graph_files.len(), ..Default::default() };
    for (path, text) in &graph_files {
        report.findings.extend(file_rule::check_file(path, text, &base));
    }

    let shapes_text = join(&shape_files);
    let joined: Vec<(String, String)> =
        graph_files.into_iter().chain(run_files).chain(vocabulary).collect();
    let (findings, evaluated) = evaluate(&join(&joined), &shapes_text)?;
    report.constraints_evaluated = evaluated;
    report.findings.extend(findings);
    Ok(report)
}

/// Evaluate a shapes graph over a data graph, both as Turtle text, returning
/// the findings with the number of constraints that were evaluated.
///
/// Shapes the reader cannot read come back as [`FindingKind::Unevaluable`]
/// findings rather than silence — the reader fails closed.
pub fn evaluate(data_ttl: &str, shapes_ttl: &str) -> Result<(Vec<Finding>, usize)> {
    let extracted = shapes::extract(shapes_ttl)?;
    let mut findings = extracted.unevaluable.clone();
    findings.extend(eval::run(data_ttl, &extracted.constraints)?);
    Ok((findings, extracted.constraints.len()))
}

/// The instance's base IRI, read from `GENERATION.ttl`'s own prefix binding by
/// the RDF parser — the instance states its identity, the reader does not guess.
pub(crate) fn base_iri(dir: &Path) -> Result<String> {
    let path = dir.join("GENERATION.ttl");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| fault(&format!("cannot read {}: {e}", path.display())))?;
    prefix_binding(&text, "reg")
        .ok_or_else(|| fault(&format!("{} declares no `reg:` prefix", path.display())))
}

/// The IRI a Turtle document binds a prefix to, via the parser's own prefix map.
pub(crate) fn prefix_binding(turtle: &str, prefix: &str) -> Option<String> {
    use oxigraph::io::{RdfFormat, RdfParser};
    let mut parser = RdfParser::from_format(RdfFormat::Turtle).for_reader(turtle.as_bytes());
    for quad in &mut parser {
        quad.ok()?;
    }
    parser.prefixes().find(|(p, _)| *p == prefix).map(|(_, iri)| iri.to_string())
}

/// Every `*.ttl` beneath `dir`, sorted by path. A directory that does not
/// exist yields nothing: `runs/` appears the first time an extractor writes
/// evidence, and an instance without one is not malformed.
fn ttl_files(dir: &Path) -> Result<Vec<(String, String)>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let pattern = format!("{}/**/*.ttl", dir.display());
    let mut out = Vec::new();
    let paths = glob::glob(&pattern).map_err(|e| fault(&format!("bad glob: {e}")))?;
    for entry in paths.flatten() {
        let text = std::fs::read_to_string(&entry)
            .map_err(|e| fault(&format!("cannot read {}: {e}", entry.display())))?;
        out.push((entry.display().to_string(), text));
    }
    out.sort();
    Ok(out)
}

fn join(files: &[(String, String)]) -> String {
    files.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join("\n")
}

fn fault(message: &str) -> ProductError {
    ProductError::ConfigError(format!("registry check: {message}"))
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
