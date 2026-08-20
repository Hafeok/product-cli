//! Our own Turtle writer for proposal files — byte-stable by construction.
//!
//! The registry's authority files are never written by the oxigraph
//! serialiser (a G0 input to this session): it is not order-stable and it
//! re-mints blank-node labels, so two runs over the same graph can differ in
//! bytes without differing in meaning. This writer emits a fixed field
//! order, sorted evidence, no blank nodes at all, and one assertion per file
//! — the registry's file rule. The serialiser stays what it is good for:
//! transport, inspection, a store round-trip in a test.

use super::axes::{DerivationConfidence, DomainRelevance, RelevanceBasis};
use super::rows::Provenance;
use super::triple::{escape_literal, ProposedAssertion, Term};
use super::vocab::PREFIXES;

/// Everything a file needs beside the assertion itself.
pub struct WriteContext {
    pub base_iri: String,
    /// RFC-3339 instant of the read — the Reading's `as_of`.
    pub as_of: String,
    /// The corpus's abstracted name.
    pub corpus: String,
    /// The commit the read was pinned to, in full.
    pub git_ref: String,
    /// IRI of the corpus-at-ref entity, the `wasDerivedFrom` target.
    pub corpus_iri: String,
    /// IRI of the extraction run, the `wasAttributedTo` target.
    pub run_iri: String,
}

/// The Turtle document for one proposed assertion.
pub fn assertion_file(assertion: &ProposedAssertion, ctx: &WriteContext) -> String {
    let base = &ctx.base_iri;
    let mut out = header(assertion, ctx);
    out.push_str(&prefixes(base));
    out.push('\n');
    out.push_str(&format!("reg:assertion-{}\n", assertion.id));
    let mut fields: Vec<(String, String)> = vec![
        ("a".into(), "reg:Assertion , reg:Reading , reg:GradedAssertion".into()),
        ("reg:subject".into(), shorten(&assertion.triple.subject, base)),
        ("reg:predicate".into(), shorten(&assertion.triple.predicate, base)),
        ("reg:object".into(), object_term(&assertion.triple.object, base)),
        ("reg:value".into(), quoted(&assertion.triple.to_value())),
        ("reg:asOf".into(), format!("{}^^xsd:dateTime", quoted(&ctx.as_of))),
        ("reg:provenance".into(), format!("reg:{}", provenance(assertion.provenance))),
        ("reg:assurance".into(), quoted(&assertion.assurance)),
        ("reg:derivationConfidence".into(), format!("reg:{}", confidence(assertion.confidence))),
        ("reg:domainRelevance".into(), format!("reg:{}", relevance(assertion.relevance))),
        ("reg:relevanceBasis".into(), format!("reg:{}", basis(assertion.relevance_basis))),
        ("reg:templateGeneration".into(), quoted(crate::registry::TEMPLATE_VERSION)),
        ("reg:mappingRow".into(), quoted(assertion.row)),
        ("reg:sourceRef".into(), quoted(&ctx.git_ref)),
    ];
    let mut evidence: Vec<String> = assertion.evidence.clone();
    evidence.sort();
    evidence.dedup();
    for cited in evidence {
        fields.push(("reg:evidence".into(), quoted(&cited)));
    }
    fields.push(("prov:wasDerivedFrom".into(), shorten(&ctx.corpus_iri, base)));
    fields.push(("prov:wasAttributedTo".into(), shorten(&ctx.run_iri, base)));
    out.push_str(&render_fields(&fields));
    out
}

/// The comment block: what the assertion says, and what it is not.
fn header(assertion: &ProposedAssertion, ctx: &WriteContext) -> String {
    format!(
        "# {}\n#\n# Proposed — not ratified. Acceptance is this file's merge, per triple.\n\
         # Row {} · derivation confidence {} · domain relevance {} ({}) · provenance {}.\n\
         # How it was read is evidence, not ratified claim: the run sidecar under\n\
         # runs/ carries the rule, the operations, the source kinds, keyed by this id.\n\
         # Read from {} at {} by the declaration-level extractor; no model in the path.\n\n",
        assertion.triple.to_value(),
        assertion.row,
        confidence(assertion.confidence),
        relevance(assertion.relevance),
        basis(assertion.relevance_basis),
        provenance(assertion.provenance),
        ctx.corpus,
        ctx.git_ref
    )
}

/// Prefix declarations: the instance base first, then the fixed set.
fn prefixes(base: &str) -> String {
    let width = PREFIXES.iter().map(|(n, _)| n.len()).chain([3]).max().unwrap_or(4) + 1;
    let mut out = format!("@prefix {:<width$} <{base}> .\n", "reg:");
    for (name, iri) in PREFIXES {
        out.push_str(&format!("@prefix {:<width$} <{iri}> .\n", format!("{name}:")));
    }
    out
}

/// Predicate/object pairs, aligned, the last one closing the statement.
fn render_fields(fields: &[(String, String)]) -> String {
    let width = fields.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (i, (key, value)) in fields.iter().enumerate() {
        let terminator = if i + 1 == fields.len() { "." } else { ";" };
        out.push_str(&format!("    {key:<width$}  {value} {terminator}\n"));
    }
    out
}

fn object_term(term: &Term, base: &str) -> String {
    match term {
        Term::Iri(iri) => shorten(iri, base),
        Term::Literal { value, datatype } => match datatype {
            Some(dt) => format!("{}^^{}", quoted(value), shorten(dt, base)),
            None => quoted(value),
        },
    }
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", escape_literal(value))
}

/// The IRI as a prefixed name where one applies and the local part is safe,
/// otherwise the full IRI in brackets. Never a guess: an unsafe local part
/// falls back rather than being rewritten.
pub fn shorten(iri: &str, base: &str) -> String {
    if let Some(local) = iri.strip_prefix(base) {
        if safe_local(local) {
            return format!("reg:{local}");
        }
    }
    for (name, namespace) in PREFIXES {
        if let Some(local) = iri.strip_prefix(namespace) {
            if safe_local(local) {
                return format!("{name}:{local}");
            }
        }
    }
    format!("<{iri}>")
}

/// A local part that needs no escaping in Turtle.
fn safe_local(local: &str) -> bool {
    !local.is_empty()
        && local.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !local.starts_with('-')
        && !local.ends_with('-')
}

fn provenance(p: Provenance) -> &'static str {
    p.as_str()
}

fn confidence(c: DerivationConfidence) -> &'static str {
    c.as_str()
}

fn relevance(r: DomainRelevance) -> &'static str {
    r.as_str()
}

fn basis(b: RelevanceBasis) -> &'static str {
    b.as_str()
}

#[cfg(test)]
#[path = "write_tests.rs"]
mod tests;
