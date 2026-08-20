//! The graph stage — CONSTRUCT rules iterated to a fixpoint over oxigraph.
//!
//! Three mechanics are load-bearing here, each recorded as a G0 input to
//! this session:
//!
//! 1. **Parse once, execute per round.** Every rule is parsed into a
//!    `PreparedSparqlQuery` before the loop and cloned per execution, so the
//!    parse cost is paid once per rule rather than once per round. The
//!    wall-time this stage reports is what `g-dec-02`'s `revisit_if`
//!    measures against, so it is measured rather than assumed.
//! 2. **Drain before insert.** A CONSTRUCT's triple iterator borrows the
//!    store, so every solution is collected before anything is written back.
//! 3. **Nothing here is authority.** The oxigraph serialiser is never used
//!    to write a registry file; this stage hands back typed triples and the
//!    project's own writer emits them.
//!
//! What is *proposed* versus merely *entailed* follows one line: an
//! assertion is proposed when it is not recomputable from the proposal set
//! itself. The containment join is — its inputs are reference positions that
//! never enter the registry — so its output is proposed. The two transitive
//! closures are not: they recompute from proposed assertions at projection
//! build, and per PRD §3 entailments never enter the authority repo.

use std::time::Instant;

use oxigraph::io::RdfFormat;
use oxigraph::model::{Quad, Term as OxTerm};
use oxigraph::sparql::{PreparedSparqlQuery, QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::error::{ProductError, Result};

use super::facts::CorpusFacts;
use super::mint::class_iri;
use super::triple::{Term, Triple};
use super::vocab::{reg, RDFS_SUBCLASS_OF};

/// One CONSTRUCT rule in the stage.
pub struct Rule {
    pub id: &'static str,
    /// Whether the rule's output is proposed for ratification, or entailed
    /// and therefore recomputed at projection build instead.
    pub proposed: bool,
    pub construct: String,
}

/// What one rule contributed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RuleReport {
    pub id: &'static str,
    pub proposed: bool,
    pub triples: usize,
    /// How many rounds this rule was still producing new triples in.
    pub productive_rounds: usize,
}

/// The fixpoint's own record — the figure `g-dec-02` is measured against.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FixpointReport {
    /// Rounds run, the last of which added nothing.
    pub rounds: usize,
    /// Rule parses. One per rule, whatever the round count — the lever.
    pub parses: usize,
    /// Rule executions: rounds × rules.
    pub executions: usize,
    pub elapsed_ms: u128,
    pub seed_triples: usize,
    pub rules: Vec<RuleReport>,
}

impl FixpointReport {
    /// Triples the stage proposes for ratification.
    pub fn proposed(&self) -> usize {
        self.rules.iter().filter(|r| r.proposed).map(|r| r.triples).sum()
    }

    /// Triples entailed, recomputable, never written to the authority repo.
    pub fn entailed(&self) -> usize {
        self.rules.iter().filter(|r| !r.proposed).map(|r| r.triples).sum()
    }
}

/// The rule set, built against the instance's base IRI.
pub fn rules(base: &str) -> Vec<Rule> {
    let depends = reg::depends_on(base);
    vec![
        Rule {
            id: "usage-direct",
            proposed: true,
            construct: format!(
                "CONSTRUCT {{ ?owner <{depends}> ?target }} WHERE {{\n  \
                 ?site <{base}refTarget> ?target ;\n        <{base}refFile> ?file ;\n        \
                 <{base}refLine> ?line .\n  \
                 ?owner <{base}declaredIn> ?file ;\n         <{base}startLine> ?start ;\n         \
                 <{base}endLine> ?end .\n  \
                 FILTER(?line >= ?start && ?line <= ?end)\n  FILTER(?owner != ?target)\n}}"
            ),
        },
        Rule {
            id: "usage-transitive",
            proposed: false,
            construct: format!(
                "CONSTRUCT {{ ?a <{depends}> ?c }} WHERE {{\n  \
                 ?a <{depends}> ?b . ?b <{depends}> ?c .\n  FILTER(?a != ?c)\n}}"
            ),
        },
        Rule {
            id: "subclass-transitive",
            proposed: false,
            construct: format!(
                "CONSTRUCT {{ ?s <{RDFS_SUBCLASS_OF}> ?o }} WHERE {{\n  \
                 ?s <{RDFS_SUBCLASS_OF}> ?m . ?m <{RDFS_SUBCLASS_OF}> ?o .\n  \
                 FILTER(?s != ?o)\n}}"
            ),
        },
    ]
}

/// Seed Turtle for the stage: the asserted triples, plus the positional
/// facts the containment join needs.
///
/// The positional facts are scaffolding for the join and are **not**
/// proposed — nothing about a line number belongs in ratified ground.
pub fn seed(base: &str, facts: &CorpusFacts, asserted: &[Triple]) -> String {
    let mut out = String::new();
    for t in asserted {
        out.push_str(&t.to_turtle());
        out.push('\n');
    }
    for decl in &facts.declarations {
        let iri = class_iri(base, &decl.name);
        out.push_str(&format!(
            "<{iri}> <{base}declaredIn> {} ; <{base}startLine> {} ; <{base}endLine> {} .\n",
            Term::plain(decl.file.clone()).to_turtle(),
            decl.start_line,
            decl.end_line
        ));
    }
    let declared = facts.declared_names();
    for (i, site) in facts.references.iter().enumerate() {
        if !declared.contains(site.target.as_str()) {
            continue;
        }
        out.push_str(&format!(
            "<{base}refsite-{i}> <{base}refTarget> <{}> ; <{base}refFile> {} ; <{base}refLine> {} .\n",
            class_iri(base, &site.target),
            Term::plain(site.file.clone()).to_turtle(),
            site.line
        ));
    }
    out
}

/// A round adds nothing new; the loop is done. The cap is a runaway
/// backstop, never a normal exit.
const MAX_ROUNDS: usize = 64;

/// The loop's accumulating state.
struct Fixpoint {
    reports: Vec<RuleReport>,
    proposed: Vec<Triple>,
    rounds: usize,
    executions: usize,
}

impl Fixpoint {
    fn new(rules: &[Rule]) -> Self {
        Fixpoint {
            reports: rules
                .iter()
                .map(|r| RuleReport {
                    id: r.id,
                    proposed: r.proposed,
                    triples: 0,
                    productive_rounds: 0,
                })
                .collect(),
            proposed: Vec::new(),
            rounds: 0,
            executions: 0,
        }
    }

    /// Record what one rule contributed this round.
    fn record(&mut self, index: usize, rule: &Rule, added: Vec<Triple>) {
        self.executions += 1;
        if added.is_empty() {
            return;
        }
        if let Some(report) = self.reports.get_mut(index) {
            report.triples += added.len();
            report.productive_rounds += 1;
        }
        if rule.proposed {
            self.proposed.extend(added);
        }
    }
}

/// Run the stage to fixpoint, returning the triples whose rules propose them.
pub fn run(
    base: &str,
    facts: &CorpusFacts,
    asserted: &[Triple],
) -> Result<(Vec<Triple>, FixpointReport)> {
    let started = Instant::now();
    let store = load(&seed(base, facts, asserted))?;
    let rule_set = rules(base);
    // Parse once. Every later execution clones the prepared query, so the
    // parse cost does not scale with the round count.
    let prepared: Vec<PreparedSparqlQuery> =
        rule_set.iter().map(|r| prepare(&r.construct)).collect::<Result<Vec<_>>>()?;
    let mut state = Fixpoint::new(&rule_set);
    loop {
        state.rounds += 1;
        let added = round(&store, &rule_set, &prepared, &mut state)?;
        if added == 0 || state.rounds >= MAX_ROUNDS {
            break;
        }
    }
    state.proposed.sort();
    state.proposed.dedup();
    let report = FixpointReport {
        rounds: state.rounds,
        parses: prepared.len(),
        executions: state.executions,
        elapsed_ms: started.elapsed().as_millis(),
        seed_triples: store.len().unwrap_or(0),
        rules: state.reports,
    };
    Ok((state.proposed, report))
}

/// One round: every rule executed once over the store as it stands.
fn round(
    store: &Store,
    rules: &[Rule],
    prepared: &[PreparedSparqlQuery],
    state: &mut Fixpoint,
) -> Result<usize> {
    let mut added_this_round = 0usize;
    for (index, (rule, query)) in rules.iter().zip(prepared).enumerate() {
        // Drain before insert: the iterator borrows the store.
        let drained = construct(query, store)?;
        let added = insert_new(store, &drained)?;
        added_this_round += added.len();
        state.record(index, rule, added);
    }
    Ok(added_this_round)
}

fn load(ttl: &str) -> Result<Store> {
    let store = Store::new().map_err(fault)?;
    store.load_from_reader(RdfFormat::Turtle, ttl.as_bytes()).map_err(fault)?;
    Ok(store)
}

fn prepare(construct: &str) -> Result<PreparedSparqlQuery> {
    SparqlEvaluator::new().parse_query(construct).map_err(fault)
}

/// Execute a prepared CONSTRUCT, collecting every triple before returning —
/// the iterator holds a borrow of the store that an insert would conflict
/// with.
fn construct(query: &PreparedSparqlQuery, store: &Store) -> Result<Vec<Quad>> {
    let results = query.clone().on_store(store).execute().map_err(fault)?;
    let QueryResults::Graph(triples) = results else { return Ok(Vec::new()) };
    let mut drained = Vec::new();
    for triple in triples {
        let triple = triple.map_err(fault)?;
        drained.push(triple.in_graph(oxigraph::model::GraphName::DefaultGraph));
    }
    Ok(drained)
}

/// Insert the quads the store does not already hold, returning the new ones.
fn insert_new(store: &Store, quads: &[Quad]) -> Result<Vec<Triple>> {
    let mut fresh = Vec::new();
    for quad in quads {
        if store.contains(quad).map_err(fault)? {
            continue;
        }
        store.insert(quad).map_err(fault)?;
        if let Some(t) = to_triple(quad) {
            fresh.push(t);
        }
    }
    Ok(fresh)
}

/// Convert back to the project's own triple type. A quad whose subject or
/// object is a blank node is dropped: identity that cannot survive a rebuild
/// has no place in a proposal.
fn to_triple(quad: &Quad) -> Option<Triple> {
    let subject = match &quad.subject {
        oxigraph::model::NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
        _ => return None,
    };
    let object = match &quad.object {
        OxTerm::NamedNode(n) => Term::iri(n.as_str()),
        OxTerm::Literal(l) => Term::Literal {
            value: l.value().to_string(),
            datatype: named_datatype(l),
        },
        _ => return None,
    };
    Some(Triple::new(subject, quad.predicate.as_str(), object))
}

/// The literal's datatype, `None` for a plain string.
fn named_datatype(literal: &oxigraph::model::Literal) -> Option<String> {
    let dt = literal.datatype().as_str().to_string();
    (dt != "http://www.w3.org/2001/XMLSchema#string").then_some(dt)
}

fn fault(e: impl std::fmt::Display) -> ProductError {
    ProductError::ConfigError(format!("ground entailment stage: {e}"))
}

#[cfg(test)]
#[path = "entail_tests.rs"]
mod tests;
