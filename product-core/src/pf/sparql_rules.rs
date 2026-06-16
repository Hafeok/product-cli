//! SPARQL rule runner — executes `sh:sparql`-style constraints via oxigraph.
//!
//! A [`SparqlRule`] is a SELECT over a Turtle projection that returns one row
//! per violation (zero rows = conformant), mirroring a `sh:SPARQLConstraint`.
//! The rule sets live in `rules_how`/`rules_what`; this module only loads a
//! projection into an oxigraph store, runs the queries, renders the rows.

use std::collections::BTreeMap;

use oxigraph::io::RdfFormat;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

use super::validate::Violation;

/// A named conformance rule: a SPARQL SELECT plus how to render each result row
/// as a [`Violation`].
pub struct SparqlRule {
    /// Diagnostic name (used for the synthetic engine-error violation).
    pub id: &'static str,
    /// Which SELECT variable binds the focus node of each violation.
    pub focus_var: &'static str,
    /// The property path reported on each violation.
    pub path: &'static str,
    /// `"violation"` (blocking) or `"warning"`.
    pub severity: &'static str,
    /// The SELECT body — full IRIs, no prefix declarations.
    pub select: &'static str,
    /// Render a violation message from a row's local-name bindings.
    pub message: fn(&BTreeMap<String, String>) -> String,
}

/// Load `ttl` into an oxigraph store and run every rule, returning one
/// [`Violation`] per result row. An engine error (malformed projection, bad
/// query) becomes a single diagnostic warning rather than a panic — these run
/// best-effort over a projection.
pub fn run_rules(ttl: &str, rules: &[SparqlRule]) -> Vec<Violation> {
    let store = match load(ttl) {
        Ok(s) => s,
        Err(e) => return vec![engine_error("load", &e)],
    };
    let mut out = Vec::new();
    for rule in rules {
        match run_one(&store, rule) {
            Ok(mut vs) => out.append(&mut vs),
            Err(e) => out.push(engine_error(rule.id, &e)),
        }
    }
    out
}

fn load(ttl: &str) -> Result<Store, String> {
    let store = Store::new().map_err(|e| e.to_string())?;
    store
        .load_from_reader(RdfFormat::Turtle, ttl.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(store)
}

fn run_one(store: &Store, rule: &SparqlRule) -> Result<Vec<Violation>, String> {
    let QueryResults::Solutions(solutions) = store.query(rule.select).map_err(|e| e.to_string())? else {
        return Ok(Vec::new());
    };
    let vars: Vec<String> = solutions.variables().iter().map(|v| v.as_str().to_string()).collect();
    let mut out = Vec::new();
    for sol in solutions {
        let sol = sol.map_err(|e| e.to_string())?;
        let mut row = BTreeMap::new();
        for var in &vars {
            if let Some(term) = sol.get(var.as_str()) {
                row.insert(var.clone(), local_name(&term.to_string()));
            }
        }
        out.push(Violation {
            focus: row.get(rule.focus_var).cloned().unwrap_or_default(),
            path: rule.path.to_string(),
            message: (rule.message)(&row),
            severity: rule.severity.to_string(),
        });
    }
    Ok(out)
}

/// Reduce a term string (`<https://…#id>`) to its readable local name — the
/// part after the last `#` or `/`, stripped of surrounding `<>`.
fn local_name(term: &str) -> String {
    let t = term.trim_start_matches('<').trim_end_matches('>');
    t.rsplit(['#', '/']).next().unwrap_or(t).to_string()
}

fn engine_error(rule: &str, e: &str) -> Violation {
    Violation {
        focus: rule.to_string(),
        path: "sparql".to_string(),
        message: format!("graph rule '{rule}' could not run: {e}"),
        severity: "warning".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial rule used to exercise the runner without depending on the
    /// real rule sets.
    const PROBE: SparqlRule = SparqlRule {
        id: "probe",
        focus_var: "s",
        path: "p",
        severity: "violation",
        select: "SELECT ?s WHERE { ?s a <https://productframework.org/ns#Thing> }",
        message: |_| "probe fired".to_string(),
    };

    #[test]
    fn runs_a_rule_and_renders_each_row() {
        let ttl = "@prefix pf: <https://productframework.org/ns#> .\n@prefix d: <https://productframework.org/x#> .\nd:A a pf:Thing .\nd:B a pf:Thing .\n";
        let vs = run_rules(ttl, &[PROBE]);
        assert_eq!(vs.len(), 2);
        assert_eq!(vs[0].path, "p");
        assert!(vs.iter().any(|v| v.focus == "A"));
    }

    #[test]
    fn malformed_turtle_degrades_to_a_warning() {
        let vs = run_rules("this is not turtle", &[PROBE]);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0].severity, "warning");
    }

    #[test]
    fn local_name_strips_namespace() {
        assert_eq!(local_name("<https://productframework.org/ns#Foo>"), "Foo");
        assert_eq!(local_name("<https://example.org/a/b/c>"), "c");
    }
}
