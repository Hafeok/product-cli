//! Evaluating extracted constraints as SPARQL over an instance's data graph.
//!
//! Each constraint compiles to one SELECT whose every result row is a
//! violation — the same shape `pf::sparql_rules` already runs the framework's
//! own graph rules in, so oxigraph stays the single query engine.

use crate::error::{ProductError, Result};

use super::shapes::{local, Constraint, Rule};
use super::{Finding, FindingKind};

/// Run every constraint over the data graph, returning one finding per row.
pub(crate) fn run(data_ttl: &str, constraints: &[Constraint]) -> Result<Vec<Finding>> {
    let mut out = Vec::new();
    for c in constraints {
        let query = compile(c);
        let rows = crate::pf::sparql_rules::select(data_ttl, &query).map_err(|e| {
            ProductError::ConfigError(format!(
                "registry check: constraint on {} could not run: {e}",
                local(&c.shape)
            ))
        })?;
        for row in rows {
            let focus = row.get("this").cloned().unwrap_or_else(|| local(&c.shape));
            out.push(Finding {
                kind: FindingKind::ShapeViolation,
                file: None,
                focus: local(&focus),
                message: describe(c),
            });
        }
    }
    Ok(out)
}

/// The SELECT one constraint becomes.
pub(crate) fn compile(c: &Constraint) -> String {
    let class = &c.class;
    let path = c.path.clone().unwrap_or_default();
    match &c.rule {
        Rule::Sparql(select) => select.clone(),
        Rule::MinCount(n) => format!(
            "SELECT ?this WHERE {{ ?this a {class} . OPTIONAL {{ ?this {path} ?v }} }} \
             GROUP BY ?this HAVING (COUNT(?v) < {n})"
        ),
        Rule::MaxCount(n) => format!(
            "SELECT ?this WHERE {{ ?this a {class} . OPTIONAL {{ ?this {path} ?v }} }} \
             GROUP BY ?this HAVING (COUNT(?v) > {n})"
        ),
        Rule::NodeKind(kind) => {
            let test = match kind.as_str() {
                "IRI" => "!isIRI(?v)",
                "Literal" => "!isLiteral(?v)",
                _ => "!isBlank(?v)",
            };
            format!("SELECT ?this WHERE {{ ?this a {class} ; {path} ?v . FILTER({test}) }}")
        }
        Rule::Datatype(dt) => format!(
            "SELECT ?this WHERE {{ ?this a {class} ; {path} ?v . \
             FILTER(!isLiteral(?v) || datatype(?v) != {dt}) }}"
        ),
        Rule::In(members) => format!(
            "SELECT ?this WHERE {{ ?this a {class} ; {path} ?v . FILTER(?v NOT IN ({})) }}",
            members.join(", ")
        ),
    }
}

/// What a constraint says when it is violated — the shape's own `sh:message`
/// where it has one, otherwise a rendering of the constraint itself.
fn describe(c: &Constraint) -> String {
    if !c.message.is_empty() {
        return c.message.clone();
    }
    let path = c.path.as_deref().map(local).unwrap_or_default();
    match &c.rule {
        Rule::MinCount(n) => format!("{path}: fewer than {n} value(s)"),
        Rule::MaxCount(n) => format!("{path}: more than {n} value(s)"),
        Rule::NodeKind(k) => format!("{path}: a value is not a {k}"),
        Rule::Datatype(dt) => format!("{path}: a value is not a {}", local(dt)),
        Rule::In(_) => format!("{path}: a value is outside the permitted set"),
        Rule::Sparql(_) => format!("{}: SPARQL constraint violated", local(&c.shape)),
    }
}
