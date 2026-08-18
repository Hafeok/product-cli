//! Reading a defined subset of SHACL out of an instance's shapes files.
//!
//! The subset is: `sh:NodeShape` with `sh:targetClass`, `sh:property` carrying
//! `sh:path` with `sh:minCount` / `sh:maxCount` / `sh:nodeKind` / `sh:datatype`
//! / `sh:in` / `sh:message`, plus `sh:sparql` with `sh:select`. Every other
//! SHACL predicate encountered makes the check fail as *unevaluable* — the
//! reader never passes a shape it did not read.

use crate::error::{ProductError, Result};

use super::{Finding, FindingKind};

const SH: &str = "http://www.w3.org/ns/shacl#";
const SUPPORTED: &[&str] = &[
    "targetClass", "property", "sparql", "path", "minCount", "maxCount", "nodeKind", "datatype",
    "in", "message", "select", "prefixes",
];

/// One evaluable constraint.
#[derive(Debug, Clone)]
pub(crate) struct Constraint {
    /// The shape it came from.
    pub shape: String,
    /// The class it targets.
    pub class: String,
    /// The property path, where the constraint has one.
    pub path: Option<String>,
    /// What it asserts.
    pub rule: Rule,
    /// The shape's own message.
    pub message: String,
}

/// The constraint kinds the reader evaluates.
#[derive(Debug, Clone)]
pub(crate) enum Rule {
    /// At least this many values.
    MinCount(u64),
    /// At most this many values.
    MaxCount(u64),
    /// Every value is an IRI, a literal, or a blank node.
    NodeKind(String),
    /// Every value carries this datatype.
    Datatype(String),
    /// Every value is drawn from this set.
    In(Vec<String>),
    /// A `sh:SPARQLConstraint` — the select returns one row per violation.
    Sparql(String),
}

/// What a shapes graph yielded.
#[derive(Debug, Default)]
pub(crate) struct Extracted {
    /// The constraints the reader will evaluate.
    pub constraints: Vec<Constraint>,
    /// Shapes it refused to evaluate.
    pub unevaluable: Vec<Finding>,
}

/// Read `shapes_ttl`, returning what can be evaluated with what cannot.
pub(crate) fn extract(shapes_ttl: &str) -> Result<Extracted> {
    let mut out = Extracted { unevaluable: unsupported_predicates(shapes_ttl)?, ..Default::default() };
    out.constraints.extend(property_constraints(shapes_ttl, &mut out.unevaluable)?);
    out.constraints.extend(sparql_constraints(shapes_ttl)?);
    Ok(out)
}

/// Fail-closed guard: every SHACL predicate used must be one the reader knows.
fn unsupported_predicates(ttl: &str) -> Result<Vec<Finding>> {
    let q = format!(
        "SELECT DISTINCT ?p WHERE {{ ?s ?p ?o . FILTER(STRSTARTS(STR(?p), \"{SH}\")) }}"
    );
    let rows = select(ttl, &q)?;
    Ok(rows
        .iter()
        .filter_map(|r| r.get("p").map(|p| local(p)))
        .filter(|name| !SUPPORTED.contains(&name.as_str()))
        .map(|name| unevaluable(&format!("sh:{name}"), "unsupported SHACL construct — the shape was not evaluated"))
        .collect())
}

/// The property-shape constraints, one per constraint kind.
fn property_constraints(ttl: &str, unread: &mut Vec<Finding>) -> Result<Vec<Constraint>> {
    let q = format!(
        "PREFIX sh: <{SH}>
         SELECT ?shape ?class ?path ?minCount ?maxCount ?nodeKind ?datatype ?message WHERE {{
            ?shape sh:targetClass ?class ; sh:property ?p . ?p sh:path ?path .
            OPTIONAL {{ ?p sh:minCount ?minCount }} OPTIONAL {{ ?p sh:maxCount ?maxCount }}
            OPTIONAL {{ ?p sh:nodeKind ?nodeKind }} OPTIONAL {{ ?p sh:datatype ?datatype }}
            OPTIONAL {{ ?p sh:message ?message }} }}"
    );
    let members = in_members(ttl)?;
    let mut out = Vec::new();
    for row in select(ttl, &q)? {
        let (shape, class, path) = match (row.get("shape"), row.get("class"), row.get("path")) {
            (Some(s), Some(c), Some(p)) => (s.clone(), c.clone(), p.clone()),
            _ => continue,
        };
        if iri(&path).is_none() {
            unread.push(unevaluable(&shape, "sh:path is not a plain IRI — property paths are not evaluated"));
            continue;
        }
        let message = row.get("message").map(|m| lexical(m)).unwrap_or_default();
        let base = Constraint { shape, class, path: Some(path.clone()), rule: Rule::MinCount(0), message };
        push_kinds(&row, &members, &path, base, &mut out, unread);
    }
    Ok(out)
}

/// Turn one result row into one constraint per kind it carries.
fn push_kinds(
    row: &std::collections::BTreeMap<String, String>,
    members: &std::collections::BTreeMap<String, Vec<String>>,
    path: &str,
    base: Constraint,
    out: &mut Vec<Constraint>,
    unread: &mut Vec<Finding>,
) {
    let mut with = |rule: Rule| out.push(Constraint { rule, ..base.clone() });
    if let Some(n) = row.get("minCount").and_then(|v| lexical(v).parse().ok()) {
        with(Rule::MinCount(n));
    }
    if let Some(n) = row.get("maxCount").and_then(|v| lexical(v).parse().ok()) {
        with(Rule::MaxCount(n));
    }
    if let Some(k) = row.get("nodeKind").map(|v| local(v)) {
        if ["IRI", "Literal", "BlankNode"].contains(&k.as_str()) {
            with(Rule::NodeKind(k));
        } else {
            unread.push(unevaluable(&base.shape, &format!("sh:nodeKind sh:{k} is not evaluated")));
        }
    }
    if let Some(dt) = row.get("datatype").cloned() {
        with(Rule::Datatype(dt));
    }
    if let Some(list) = members.get(&key(&base.shape, path)) {
        with(Rule::In(list.clone()));
    }
}

/// `sh:in` membership, keyed by shape + path so no blank-node label is relied on.
fn in_members(ttl: &str) -> Result<std::collections::BTreeMap<String, Vec<String>>> {
    let q = format!(
        "PREFIX sh: <{SH}>
         SELECT ?shape ?path ?member WHERE {{
            ?shape sh:property ?p . ?p sh:path ?path ; sh:in/<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>*/<http://www.w3.org/1999/02/22-rdf-syntax-ns#first> ?member }}"
    );
    let mut map: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for row in select(ttl, &q)? {
        if let (Some(s), Some(p), Some(m)) = (row.get("shape"), row.get("path"), row.get("member")) {
            map.entry(key(s, p)).or_default().push(m.clone());
        }
    }
    Ok(map)
}

/// The `sh:sparql` constraints — a select returning one row per violation.
fn sparql_constraints(ttl: &str) -> Result<Vec<Constraint>> {
    let q = format!(
        "PREFIX sh: <{SH}>
         SELECT ?shape ?class ?select ?message WHERE {{
            ?shape sh:targetClass ?class ; sh:sparql ?c . ?c sh:select ?select .
            OPTIONAL {{ ?c sh:message ?message }} }}"
    );
    Ok(select(ttl, &q)?
        .iter()
        .filter_map(|row| {
            Some(Constraint {
                shape: row.get("shape")?.clone(),
                class: row.get("class")?.clone(),
                path: None,
                rule: Rule::Sparql(lexical(row.get("select")?)),
                message: row.get("message").map(|m| lexical(m)).unwrap_or_default(),
            })
        })
        .collect())
}

fn key(shape: &str, path: &str) -> String {
    format!("{shape} {path}")
}

fn unevaluable(focus: &str, message: &str) -> Finding {
    Finding {
        kind: FindingKind::Unevaluable,
        file: None,
        focus: focus.to_string(),
        message: message.to_string(),
    }
}

fn select(
    ttl: &str,
    query: &str,
) -> Result<Vec<std::collections::BTreeMap<String, String>>> {
    crate::pf::sparql_rules::select(ttl, query)
        .map_err(|e| ProductError::ConfigError(format!("registry check: shapes could not be read: {e}")))
}

/// The IRI inside a `<…>` term, if it is one.
pub(crate) fn iri(term: &str) -> Option<&str> {
    term.strip_prefix('<')?.strip_suffix('>')
}

/// A term's local name — after the last `#` or `/`.
pub(crate) fn local(term: &str) -> String {
    let t = term.trim_start_matches('<').trim_end_matches('>');
    t.rsplit(['#', '/']).next().unwrap_or(t).to_string()
}

/// A literal term's lexical form, without quotes, datatype, language tag.
pub(crate) fn lexical(term: &str) -> String {
    let t = term.trim();
    if !t.starts_with('"') {
        return t.to_string();
    }
    match t.rfind('"') {
        Some(end) if end > 0 => t[1..end].replace("\\\"", "\"").replace("\\n", "\n"),
        _ => t.to_string(),
    }
}
