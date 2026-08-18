//! Planning one extraction run — facts in, proposal files out.
//!
//! Pure: no I/O, no clock of its own (the `as_of` is supplied so a run is
//! reproducible), no network. Everything the reviewer will see is decided
//! here, including which rows fired, which found nothing, which were gated
//! off by the capability probe, and which are not derivable at all.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::error::Result;
use crate::registry::{evaluate, Finding};

use super::declared::{self, DeclaredNotes};
use super::entail::{self, FixpointReport};
use super::facts::CorpusFacts;
use super::mint::{corpus_iri, run_iri};
use super::propose::{cite, Proposer};
use super::rows::{row, Outcome, Row, FOREIGN_KEY_REASON, ROWS};
use super::structural;
use super::triple::{ProposedAssertion, Triple};
use super::write::{assertion_file, WriteContext};

/// What the caller supplies that the facts do not carry.
#[derive(Debug, Clone)]
pub struct ExtractionParams {
    /// The instance's base IRI, read from its own `GENERATION.ttl`.
    pub base_iri: String,
    /// RFC-3339 instant of the read.
    pub as_of: String,
    /// Directory inside the instance the files land in.
    pub graph_dir: String,
}

/// One file the run would write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedFile {
    /// Instance-relative path.
    pub path: String,
    pub contents: String,
}

/// One row's fate on this corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowResult {
    pub row: Row,
    pub outcome: Outcome,
}

/// Everything one run proposes, with the record of how it got there.
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionPlan {
    pub corpus: String,
    pub git_ref: String,
    pub assertions: Vec<ProposedAssertion>,
    pub rows: Vec<RowResult>,
    pub fixpoint: FixpointReport,
    pub files: Vec<PlannedFile>,
    /// SHACL findings against the instance's own shapes. A non-empty list
    /// means the proposal must not be written.
    pub shacl: Vec<Finding>,
    pub constraints_evaluated: usize,
    /// Assertions the canonical graph already holds, so this run proposes
    /// nothing for them.
    pub already_ratified: usize,
    #[serde(skip)]
    pub notes: DeclaredNotes,
}

impl ExtractionPlan {
    /// Is the proposal fit to write?
    pub fn conformant(&self) -> bool {
        self.shacl.is_empty()
    }

    /// Proposed assertion counts per assurance grade.
    pub fn by_grade(&self) -> Vec<(&'static str, usize)> {
        let mut out: Vec<(&'static str, usize)> = Vec::new();
        for a in &self.assertions {
            let key = a.grade.as_str();
            match out.iter_mut().find(|(k, _)| *k == key) {
                Some((_, n)) => *n += 1,
                None => out.push((key, 1)),
            }
        }
        out.sort_by_key(|(k, _)| *k);
        out
    }
}

/// Plan the run.
///
/// `ratified` holds the assertion ids the canonical graph already carries.
/// An id in that set is dropped rather than re-proposed: identity is
/// content-derived, so an already-ratified triple read again is the *same*
/// assertion, and re-emitting it would rewrite ratified content rather than
/// add to it.
pub fn plan_extraction(
    facts: &CorpusFacts,
    params: &ExtractionParams,
    shapes_ttl: &str,
    ratified: &BTreeSet<String>,
) -> Result<ExtractionPlan> {
    let base = &params.base_iri;
    let proposer = Proposer::new(&facts.instrument);
    let mut notes = DeclaredNotes::default();
    let mut rows: Vec<RowResult> = Vec::new();
    let mut assertions: Vec<ProposedAssertion> = Vec::new();

    let (classes, outcome) = declared::classes(base, facts, &proposer);
    push_row(&mut rows, "classes", outcome);
    assertions.extend(classes);

    let (subclass, outcome) = declared::subclass(base, facts, &proposer, &mut notes);
    push_row(&mut rows, "subclass", outcome);
    assertions.extend(subclass);

    let (properties, outcome) = declared::properties(base, facts, &proposer, &mut notes);
    push_row(&mut rows, "properties", outcome);
    assertions.extend(properties);

    push_row(&mut rows, "foreign-keys", Outcome::NotDerivable { reason: FOREIGN_KEY_REASON });

    let (modules, outcome) = structural::modules(base, facts, &proposer);
    push_row(&mut rows, "modules", outcome);
    assertions.extend(modules);

    let (synonyms, outcome) = structural::synonyms(base, facts, &proposer);
    push_row(&mut rows, "synonyms", outcome);
    assertions.extend(synonyms);

    let (usage, fixpoint) = usage_row(base, facts, &proposer, &assertions, &mut rows)?;
    assertions.extend(usage);

    let assertions = drop_ratified(declared::dedup(assertions), ratified);
    let already_ratified = ratified.len();
    let files = render(&assertions, facts, params);
    let (shacl, constraints_evaluated) = check(&files, shapes_ttl)?;
    Ok(ExtractionPlan {
        corpus: facts.corpus.clone(),
        git_ref: facts.git_ref.clone(),
        assertions,
        rows,
        fixpoint,
        files,
        shacl,
        constraints_evaluated,
        already_ratified,
        notes,
    })
}

/// The usage row runs the graph stage over what the declared rows asserted.
fn usage_row(
    base: &str,
    facts: &CorpusFacts,
    proposer: &Proposer,
    asserted: &[ProposedAssertion],
    rows: &mut Vec<RowResult>,
) -> Result<(Vec<ProposedAssertion>, FixpointReport)> {
    let Some(r) = row("usage-relations") else {
        return Ok((Vec::new(), empty_fixpoint()));
    };
    if let Some(blocked) = declared::gate(facts, r) {
        push_row(rows, "usage-relations", blocked);
        return Ok((Vec::new(), empty_fixpoint()));
    }
    let seed: Vec<Triple> = asserted.iter().map(|a| a.triple.clone()).collect();
    let (derived, fixpoint) = entail::run(base, facts, &seed)?;
    let proposals: Vec<ProposedAssertion> = derived
        .into_iter()
        .map(|t| {
            let evidence = usage_evidence(base, facts, &t);
            proposer.propose(r, t, evidence)
        })
        .collect();
    let (proposals, outcome) = declared::settle(proposals);
    push_row(rows, "usage-relations", outcome);
    Ok((proposals, fixpoint))
}

/// The reference sites behind one derived edge.
///
/// An inferred edge with no citable position is a triple a reviewer can only
/// rubber-stamp, so the rule's output is traced back to the reads that
/// produced it: the use sites of the target that fall inside the subject's
/// declared span.
fn usage_evidence(base: &str, facts: &CorpusFacts, triple: &Triple) -> Vec<String> {
    let Some(owner) = declaration_named(base, facts, &triple.subject) else {
        return vec![DERIVED_BY_RULE.to_string()];
    };
    let target = match &triple.object {
        crate::ground::triple::Term::Iri(iri) => declaration_named(base, facts, iri),
        _ => None,
    };
    let Some(target) = target else { return vec![DERIVED_BY_RULE.to_string()] };
    let mut sites: Vec<String> = facts
        .references
        .iter()
        .filter(|s| s.target == target.name && s.file == owner.file)
        .filter(|s| owner.start_line <= s.line && s.line <= owner.end_line)
        .map(|s| cite(&s.file, s.line))
        .collect();
    sites.sort();
    sites.dedup();
    sites.insert(0, DERIVED_BY_RULE.to_string());
    sites
}

/// What the assurance field cannot say on its own: the rule, named.
const DERIVED_BY_RULE: &str = "derived by CONSTRUCT over reference edges, to fixpoint";

/// The declaration an IRI under the instance base refers to.
fn declaration_named<'a>(
    base: &str,
    facts: &'a CorpusFacts,
    iri: &str,
) -> Option<&'a crate::ground::facts::Declaration> {
    let local = iri.strip_prefix(base)?;
    facts.declarations.iter().find(|d| crate::ground::mint::local_name(&d.name) == local)
}

fn empty_fixpoint() -> FixpointReport {
    FixpointReport {
        rounds: 0,
        parses: 0,
        executions: 0,
        elapsed_ms: 0,
        seed_triples: 0,
        rules: Vec::new(),
    }
}

fn push_row(rows: &mut Vec<RowResult>, id: &str, outcome: Outcome) {
    if let Some(r) = ROWS.iter().find(|r| r.id == id) {
        rows.push(RowResult { row: *r, outcome });
    }
}

/// Drop what the canonical graph already ratified.
fn drop_ratified(
    assertions: Vec<ProposedAssertion>,
    ratified: &BTreeSet<String>,
) -> Vec<ProposedAssertion> {
    assertions.into_iter().filter(|a| !ratified.contains(&a.id)).collect()
}

/// Render one file per assertion, sorted by path so a run is reproducible.
fn render(
    assertions: &[ProposedAssertion],
    facts: &CorpusFacts,
    params: &ExtractionParams,
) -> Vec<PlannedFile> {
    let ctx = WriteContext {
        base_iri: params.base_iri.clone(),
        as_of: params.as_of.clone(),
        corpus: facts.corpus.clone(),
        git_ref: facts.git_ref.clone(),
        corpus_iri: corpus_iri(&params.base_iri, &facts.corpus, &facts.git_ref),
        run_iri: run_iri(&params.base_iri, &facts.corpus, &facts.git_ref),
    };
    let mut files: Vec<PlannedFile> = assertions
        .iter()
        .map(|a| PlannedFile {
            path: format!("{}/{}", params.graph_dir, a.file_name()),
            contents: assertion_file(a, &ctx),
        })
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

/// Run the instance's own shapes over the proposal, before anything is written.
fn check(files: &[PlannedFile], shapes_ttl: &str) -> Result<(Vec<Finding>, usize)> {
    if files.is_empty() || shapes_ttl.trim().is_empty() {
        return Ok((Vec::new(), 0));
    }
    let data: String = files.iter().map(|f| f.contents.as_str()).collect::<Vec<_>>().join("\n");
    evaluate(&data, shapes_ttl)
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
