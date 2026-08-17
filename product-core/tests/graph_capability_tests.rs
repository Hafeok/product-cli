//! Graph-engine capability tests for the G-track's projection build.
//!
//! Each test pins one oxigraph capability the G-track depends on, so a future
//! dependency move cannot silently withdraw it: named-graph write/read/
//! isolation (source graphs per extraction run, canonical vs proposal
//! separation), SPARQL 1.1 Query including CONSTRUCT (g-dec-02's primary
//! reasoner route), SPARQL 1.1 Update (the projection build), Turtle
//! round-tripping (the registry's per-triple file format), plus a
//! CONSTRUCT-fixpoint smoke run.
//!
//! These live beside the `pf` suite rather than inside it: no `pf` module
//! writes named graphs or runs Update today — g-dec-02's reasoner is G0
//! proper — so there is no slice to attach them to.
//!
//! The fixpoint case is deliberately a smoke test, not a benchmark. It
//! establishes that the shape works and terminates before G0 scale, and
//! reports a wall-time figure for g-dec-02's `revisit_if` to measure
//! against later. It parses each rule once per round, matching what
//! `pf::sparql_rules` does today, so the figure describes the un-optimised
//! route rather than the `SparqlEvaluator::prepare` one.

use std::time::Instant;

use oxigraph::io::RdfFormat;
use oxigraph::model::{GraphName, GraphNameRef, NamedNodeRef, QuadRef, Term};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

const EX: &str = "http://example.org/";

fn store_from(ttl: &str) -> Store {
    let store = Store::new().expect("oxigraph store");
    store
        .load_from_reader(RdfFormat::Turtle, ttl.as_bytes())
        .expect("load turtle");
    store
}

fn iri(local: &str) -> NamedNodeRef<'static> {
    // Leaked so the ref can be 'static; test-only, bounded by the case count.
    let s: &'static str = Box::leak(format!("{EX}{local}").into_boxed_str());
    NamedNodeRef::new(s).expect("valid IRI")
}

fn query(store: &Store, q: &str) -> QueryResults<'static> {
    SparqlEvaluator::new()
        .parse_query(q)
        .expect("parse query")
        .on_store(store)
        .execute()
        .expect("execute query")
}

/// Every row's bindings, keyed by variable, as raw term strings.
fn rows(store: &Store, q: &str) -> Vec<Vec<(String, String)>> {
    let QueryResults::Solutions(solutions) = query(store, q) else {
        panic!("expected SELECT solutions");
    };
    let vars: Vec<String> = solutions.variables().iter().map(|v| v.as_str().to_string()).collect();
    let mut out = Vec::new();
    for sol in solutions {
        let sol = sol.expect("solution row");
        let mut row = Vec::new();
        for var in &vars {
            if let Some(term) = sol.get(var.as_str()) {
                row.push((var.clone(), term.to_string()));
            }
        }
        out.push(row);
    }
    out
}

fn ask(store: &Store, q: &str) -> bool {
    let QueryResults::Boolean(b) = query(store, q) else {
        panic!("expected ASK boolean");
    };
    b
}

/// Materialise a CONSTRUCT into an owned triple vector.
fn construct(store: &Store, q: &str) -> Vec<oxigraph::model::Triple> {
    let QueryResults::Graph(triples) = query(store, q) else {
        panic!("expected CONSTRUCT graph");
    };
    triples.map(|t| t.expect("construct triple")).collect()
}

// ---------------------------------------------------------------------------
// Capability 1 — named graph write, read, isolation
// ---------------------------------------------------------------------------

#[test]
fn named_graphs_are_written_read_back_and_isolated() {
    let store = Store::new().expect("store");
    let (run_a, run_b) = (iri("run/a"), iri("run/b"));
    let (p, subj) = (iri("states"), iri("s"));

    store.insert(QuadRef::new(subj, p, iri("only-in-a"), GraphNameRef::from(run_a))).expect("insert a");
    store.insert(QuadRef::new(subj, p, iri("only-in-b"), GraphNameRef::from(run_b))).expect("insert b");
    store.insert(QuadRef::new(subj, p, iri("in-default"), GraphNameRef::DefaultGraph)).expect("insert default");

    // Read back: each named graph holds exactly its own quad.
    assert_eq!(store.quads_for_pattern(None, None, None, Some(run_a.into())).count(), 1, "run/a holds one quad");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(run_b.into())).count(), 1, "run/b holds one quad");
    assert_eq!(store.len().expect("len"), 3, "three quads across three graphs");

    // The store enumerates both named graphs, and not the default graph.
    let mut graphs: Vec<String> = store
        .named_graphs()
        .map(|g| g.expect("named graph").to_string())
        .collect();
    graphs.sort();
    assert_eq!(graphs, vec![format!("<{EX}run/a>"), format!("<{EX}run/b>")]);

    // Isolation: a GRAPH-scoped query sees one run's triples only.
    let a_rows = rows(&store, &format!("SELECT ?o WHERE {{ GRAPH <{EX}run/a> {{ ?s <{EX}states> ?o }} }}"));
    assert_eq!(a_rows.len(), 1);
    assert_eq!(a_rows[0][0].1, format!("<{EX}only-in-a>"), "run/a query sees only run/a");

    // Isolation: the default graph does not see named-graph triples.
    let default_rows = rows(&store, &format!("SELECT ?o WHERE {{ ?s <{EX}states> ?o }}"));
    assert_eq!(default_rows.len(), 1, "default graph is not the union of named graphs");
    assert_eq!(default_rows[0][0].1, format!("<{EX}in-default>"));

    // Isolation under deletion: dropping one run leaves the others intact.
    store.remove_named_graph(run_a).expect("drop run/a");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(run_a.into())).count(), 0, "run/a emptied");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(run_b.into())).count(), 1, "run/b survives");
    assert!(ask(&store, &format!("ASK {{ ?s <{EX}states> <{EX}in-default> }}")), "default graph survives");
}

// ---------------------------------------------------------------------------
// Capability 2 — SPARQL 1.1 Query, including CONSTRUCT
// ---------------------------------------------------------------------------

#[test]
fn sparql_query_covers_select_ask_aggregate_subquery_and_construct() {
    let ttl = format!(
        "@prefix ex: <{EX}> .\n\
         ex:a ex:kind \"entity\" ; ex:weight 3 ; ex:parent ex:b .\n\
         ex:b ex:kind \"entity\" ; ex:weight 4 .\n\
         ex:c ex:kind \"event\"  ; ex:weight 5 .\n"
    );
    let store = store_from(&ttl);

    // SELECT with FILTER and OPTIONAL.
    let selected = rows(
        &store,
        &format!("SELECT ?s ?p WHERE {{ ?s <{EX}kind> \"entity\" . OPTIONAL {{ ?s <{EX}parent> ?p }} }}"),
    );
    assert_eq!(selected.len(), 2, "two entities");
    assert!(selected.iter().any(|r| r.len() == 2), "one entity binds the OPTIONAL parent");
    assert!(selected.iter().any(|r| r.len() == 1), "the other leaves it unbound");

    // ASK.
    assert!(ask(&store, &format!("ASK {{ <{EX}a> <{EX}parent> <{EX}b> }}")));
    assert!(!ask(&store, &format!("ASK {{ <{EX}a> <{EX}parent> <{EX}c> }}")));

    // SPARQL 1.1 aggregate with GROUP BY and HAVING.
    let grouped = rows(
        &store,
        &format!(
            "SELECT ?k (COUNT(?s) AS ?n) (SUM(?w) AS ?total) WHERE \
             {{ ?s <{EX}kind> ?k ; <{EX}weight> ?w }} GROUP BY ?k HAVING (COUNT(?s) > 1)"
        ),
    );
    assert_eq!(grouped.len(), 1, "only the 'entity' group survives HAVING");
    let flat: Vec<&(String, String)> = grouped[0].iter().collect();
    assert!(flat.iter().any(|(v, val)| v == "n" && val.contains('2')), "COUNT is 2: {flat:?}");
    assert!(flat.iter().any(|(v, val)| v == "total" && val.contains('7')), "SUM is 7: {flat:?}");

    // SPARQL 1.1 sub-SELECT with BIND.
    let sub = rows(
        &store,
        &format!(
            "SELECT ?s ?doubled WHERE \
             {{ {{ SELECT ?s ?w WHERE {{ ?s <{EX}weight> ?w }} }} BIND(?w * 2 AS ?doubled) FILTER(?doubled >= 8) }}"
        ),
    );
    assert_eq!(sub.len(), 2, "weights 4 and 5 double past the filter");

    // CONSTRUCT materialises new triples without touching the source graph.
    let built = construct(
        &store,
        &format!("CONSTRUCT {{ ?p <{EX}childOf> ?s }} WHERE {{ ?s <{EX}parent> ?p }}"),
    );
    assert_eq!(built.len(), 1, "one inverse edge constructed");
    assert_eq!(built[0].predicate.as_str(), format!("{EX}childOf"));
    assert_eq!(built[0].subject.to_string(), format!("<{EX}b>"));
    assert_eq!(store.len().expect("len"), 7, "CONSTRUCT does not mutate the store");
}

// ---------------------------------------------------------------------------
// Capability 3 — SPARQL 1.1 Update
// ---------------------------------------------------------------------------

#[test]
fn sparql_update_inserts_deletes_and_rebuilds_a_named_graph() {
    let store = Store::new().expect("store");

    // INSERT DATA into a named graph.
    store
        .update(&format!(
            "INSERT DATA {{ GRAPH <{EX}proj> {{ <{EX}a> <{EX}p> \"one\" . <{EX}b> <{EX}p> \"two\" }} }}"
        ))
        .expect("insert data");
    assert_eq!(store.len().expect("len"), 2);

    // DELETE DATA removes one triple.
    store
        .update(&format!("DELETE DATA {{ GRAPH <{EX}proj> {{ <{EX}a> <{EX}p> \"one\" }} }}"))
        .expect("delete data");
    assert_eq!(store.len().expect("len"), 1);

    // DELETE/INSERT WHERE rewrites in place — the projection-build shape.
    store
        .update(&format!(
            "DELETE {{ GRAPH <{EX}proj> {{ ?s <{EX}p> ?o }} }} \
             INSERT {{ GRAPH <{EX}proj> {{ ?s <{EX}q> ?o }} }} \
             WHERE  {{ GRAPH <{EX}proj> {{ ?s <{EX}p> ?o }} }}"
        ))
        .expect("delete/insert where");
    assert!(ask(&store, &format!("ASK {{ GRAPH <{EX}proj> {{ <{EX}b> <{EX}q> \"two\" }} }}")), "predicate rewritten");
    assert!(!ask(&store, &format!("ASK {{ GRAPH <{EX}proj> {{ ?s <{EX}p> ?o }} }}")), "old predicate gone");

    // INSERT { } WHERE { } across graphs — how a projection is derived.
    store
        .update(&format!(
            "INSERT {{ GRAPH <{EX}derived> {{ ?s <{EX}derivedFrom> <{EX}proj> }} }} \
             WHERE  {{ GRAPH <{EX}proj> {{ ?s ?p ?o }} }}"
        ))
        .expect("cross-graph insert");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(iri("derived").into())).count(), 1);

    // Graph management: DROP then rebuild, leaving the sibling graph alone.
    store.update(&format!("DROP GRAPH <{EX}proj>")).expect("drop graph");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(iri("proj").into())).count(), 0, "proj dropped");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(iri("derived").into())).count(), 1, "derived intact");
    store
        .update(&format!("INSERT DATA {{ GRAPH <{EX}proj> {{ <{EX}c> <{EX}p> \"three\" }} }}"))
        .expect("rebuild graph");
    assert_eq!(store.quads_for_pattern(None, None, None, Some(iri("proj").into())).count(), 1, "proj rebuilt");
}

// ---------------------------------------------------------------------------
// Capability 4 — Turtle parse and serialise round-trip
// ---------------------------------------------------------------------------

#[test]
fn turtle_round_trips_through_parse_and_serialise() {
    let ttl = format!(
        "@prefix ex: <{EX}> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
         ex:s ex:plain \"a literal\" ;\n\
             ex:tagged \"en literal\"@en ;\n\
             ex:typed \"42\"^^xsd:integer ;\n\
             ex:iri ex:o ;\n\
             ex:blank [ ex:nested \"inner\" ] .\n"
    );
    let first = store_from(&ttl);
    assert_eq!(first.len().expect("len"), 6, "five direct triples plus the nested one");

    // Serialise the default graph back to Turtle, then reparse it.
    let out = first
        .dump_graph_to_writer(GraphNameRef::DefaultGraph, RdfFormat::Turtle, Vec::<u8>::new())
        .expect("serialise turtle");
    let out = String::from_utf8(out).expect("utf-8 turtle");
    let second = store_from(&out);
    assert_eq!(second.len().expect("len"), first.len().expect("len"), "triple count survives");

    // Content is what round-trips, not bytes: oxigraph's Turtle serialiser is
    // deliberately not order-stable — it emits in the store's internal index
    // order, so a parse -> serialise cycle reorders subjects and predicates and
    // re-mints blank-node labels, even for a blank-node-free graph. A third
    // cycle therefore has to be compared on content too. Anything in the
    // G-track needing byte-deterministic Turtle (the registry's per-triple
    // files, `ledger reindex`) must keep emitting it itself, as `pf::turtle`
    // already does, rather than reaching for `dump_graph_to_writer`.
    let third = store_from(
        &String::from_utf8(
            second
                .dump_graph_to_writer(GraphNameRef::DefaultGraph, RdfFormat::Turtle, Vec::<u8>::new())
                .expect("re-serialise"),
        )
        .expect("utf-8 turtle"),
    );
    assert_eq!(third.len().expect("len"), first.len().expect("len"), "content survives repeated cycles");

    // Every ground term survives with its lexical form and datatype/language.
    let ground = |store: &Store| -> Vec<String> {
        let mut terms: Vec<String> = store
            .quads_for_pattern(None, None, None, None)
            .filter_map(|q| q.ok())
            .filter(|q| !matches!(q.object, Term::BlankNode(_)))
            .map(|q| format!("{} {}", q.predicate, q.object))
            .collect();
        terms.sort();
        terms
    };
    assert_eq!(ground(&second), ground(&first), "ground terms round-trip unchanged");
    assert_eq!(ground(&third), ground(&first), "and stay unchanged over a second cycle");
    assert!(
        ground(&second).iter().any(|t| t.contains("\"en literal\"@en")),
        "language tag preserved: {:?}",
        ground(&second)
    );
    assert!(
        ground(&second).iter().any(|t| t.contains("XMLSchema#integer")),
        "datatype preserved: {:?}",
        ground(&second)
    );

    // The blank node survives as a blank node (its label is not stable, and
    // nothing here depends on it being).
    assert_eq!(
        second.quads_for_pattern(None, None, None, None).filter_map(|q| q.ok())
            .filter(|q| matches!(q.object, Term::BlankNode(_))).count(),
        1,
        "one blank-node object survives"
    );
}

// ---------------------------------------------------------------------------
// Capability 5 — CONSTRUCT-fixpoint smoke run
// ---------------------------------------------------------------------------

/// Run each rule as a CONSTRUCT, insert what it yields, and repeat until the
/// store stops growing. Returns the round count — the round that added
/// nothing is counted, so a saturated graph costs one round.
fn run_to_fixpoint(store: &Store, rules: &[&str], max_rounds: usize) -> usize {
    for round in 1..=max_rounds {
        let before = store.len().expect("len before round");
        for rule in rules {
            // Drained into an owned batch first: the triple iterator borrows
            // the store, so inserting mid-iteration will not borrow-check.
            for triple in construct(store, rule) {
                store
                    .insert(&triple.in_graph(GraphName::DefaultGraph))
                    .expect("insert entailed triple");
            }
        }
        if store.len().expect("len after round") == before {
            return round;
        }
    }
    panic!("fixpoint did not converge within {max_rounds} rounds");
}

#[test]
fn construct_fixpoint_converges_on_a_small_rule_set() {
    // A 4-node chain plus one sibling edge: small, but deep enough that a
    // transitive rule needs several rounds rather than one.
    let ttl = format!(
        "@prefix ex: <{EX}> .\n\
         ex:a ex:parent ex:b .\n\
         ex:b ex:parent ex:c .\n\
         ex:c ex:parent ex:d .\n\
         ex:b ex:sibling ex:e .\n"
    );
    let store = store_from(&ttl);
    let asserted = store.len().expect("len");
    assert_eq!(asserted, 4);

    let rules = [
        // Transitive closure of parent into ancestor.
        format!(
            "CONSTRUCT {{ ?x <{EX}ancestor> ?z }} WHERE \
             {{ {{ ?x <{EX}parent> ?z }} UNION {{ ?x <{EX}parent> ?y . ?y <{EX}ancestor> ?z }} }}"
        ),
        // Symmetry of sibling.
        format!("CONSTRUCT {{ ?y <{EX}sibling> ?x }} WHERE {{ ?x <{EX}sibling> ?y }}"),
    ];
    let rules: Vec<&str> = rules.iter().map(String::as_str).collect();

    let started = Instant::now();
    let rounds = run_to_fixpoint(&store, &rules, 20);
    let elapsed = started.elapsed();

    // ancestor: a→b,c,d · b→c,d · c→d = 6. sibling: b↔e adds 1. Total 4+7=11.
    let total = store.len().expect("len");
    assert_eq!(total, 11, "closure is exactly the expected 7 entailments over 4 asserted triples");
    assert!(rounds <= 5, "converged in {rounds} rounds");

    // The first datum for g-dec-02's `revisit_if`, not a benchmark: a loose
    // ceiling that only catches a pathological regression.
    assert!(
        elapsed.as_secs() < 5,
        "fixpoint smoke run took {elapsed:?}, far beyond the smoke-test ceiling"
    );
    eprintln!(
        "fixpoint smoke: {asserted} asserted -> {total} total, {} entailed, {rounds} rounds, {elapsed:?}",
        total - asserted
    );
}
