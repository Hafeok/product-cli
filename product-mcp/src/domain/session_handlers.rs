//! Handlers for the session-level domain tools (state, finalize, query).

use std::path::Path;

use product_core::pf::query;
use product_core::pf::questions::{open_questions, Focus};
use product_core::pf::session::DomainSession;
use serde_json::{json, Value};

use super::args::{opt_str, str_array};

/// `session_start` — open/create the active session, optionally seeding from
/// prior Turtle. Persists the new session and returns its state summary.
pub fn start(session_dir: &Path, a: &Value, now: String) -> Result<Value, String> {
    let product = super::args::req_str(a, "product")?;
    let title = opt_str(a, "title");
    let participants = str_array(a, "participants");
    let seed = opt_str(a, "seed_graph");
    let session = DomainSession::start(&product, title, participants, seed.as_deref(), now)
        .map_err(|e| format!("{}", e))?;
    session.save(session_dir).map_err(|e| format!("{}", e))?;
    let mut state = session.state_json();
    if let Value::Object(ref mut map) = state {
        map.insert("ok".to_string(), json!(true));
    }
    Ok(state)
}

/// `session_finalize` — validate, and on conformance write the Turtle export
/// plus the provenance record next to the session, returning both.
pub fn finalize(s: &mut DomainSession, session_dir: &Path, now: String) -> Result<Value, String> {
    let fin = s.finalize(now);
    let mut out = serde_json::to_value(&fin).map_err(|e| format!("serialize: {}", e))?;
    if fin.ok {
        let ttl = fin.turtle.clone().unwrap_or_default();
        let ttl_path = session_dir.join(format!("{}.ttl", s.product));
        product_core::fileops::write_file_atomic(&ttl_path, &ttl).map_err(|e| format!("{}", e))?;
        let prov = serde_json::to_string_pretty(&fin.provenance)
            .map_err(|e| format!("serialize provenance: {}", e))?;
        let prov_path = session_dir.join(format!("{}.provenance.json", s.product));
        product_core::fileops::write_file_atomic(&prov_path, &prov).map_err(|e| format!("{}", e))?;
        s.finalized = true;
        if let Value::Object(ref mut map) = out {
            map.insert("turtlePath".to_string(), json!(ttl_path.display().to_string()));
            map.insert("provenancePath".to_string(), json!(prov_path.display().to_string()));
        }
    }
    Ok(out)
}

/// `open_questions` — the facilitation driver, optionally limited to a half.
pub fn questions(s: &DomainSession, a: &Value) -> Value {
    let focus = opt_str(a, "focus").map(|f| Focus::parse(&f)).unwrap_or(Focus::All);
    json!({ "openQuestions": open_questions(&s.graph, focus) })
}

/// `query` — a convenience query keyed by `about`, or a raw SPARQL SELECT.
pub fn run_query(s: &DomainSession, a: &Value) -> Result<Value, String> {
    if let Some(sparql) = opt_str(a, "sparql") {
        return query::sparql(&s.graph, &s.product, &sparql).map_err(|e| format!("{}", e));
    }
    let about = opt_str(a, "about");
    let question = opt_str(a, "question");
    match (about, question) {
        (Some(id), Some(q)) => convenience(&s.graph, &id, &q),
        (Some(id), None) => query::describe(&s.graph, &id).map_err(|e| format!("{}", e)),
        _ => Err("query needs `about` (with optional `question`) or `sparql`".to_string()),
    }
}

fn convenience(graph: &product_core::pf::DomainGraph, id: &str, q: &str) -> Result<Value, String> {
    match q {
        "whatHappensTo" => Ok(query::what_happens_to(graph, id)),
        "contextContents" => Ok(query::context_contents(graph, id)),
        "entityRelations" => Ok(query::entity_relations(graph, id)),
        "flowsInContext" => Ok(query::flows_in_context(graph, id)),
        other => Err(format!("unknown query question {:?}", other)),
    }
}
