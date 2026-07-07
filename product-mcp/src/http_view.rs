//! Live-view API routes — `/api/graph`, `/api/pf`, `/api/session`, `/api/events`.
//!
//! The read-only projections the web views consume, rebuilt from `.product/`
//! on every request (the view is always derived; the `/api/events` SSE tells
//! the browser when to re-fetch). Split from the HTTP transport (`http.rs`),
//! which owns the router, the MCP endpoint, and the server lifecycle.

use std::path::Path;
use std::sync::Arc;

use product_core::author::domain::{has_graph, session_dir};
use product_core::config::ProductConfig;
use product_core::pf::deployable_unit as du;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::viz::{to_view_graph_with_how, ViewGraph};
use product_core::pf::workflow::{workflow_path, WorkflowSession};

use super::http::AppState;

/// A `?session=<id>` query selecting which session the view follows, and a
/// `?product=<name>` query selecting which product to render (e.g. the acme
/// showcase alongside the self-hosted product-cli).
#[derive(serde::Deserialize, Default)]
pub(crate) struct SessionQuery {
    session: Option<String>,
    product: Option<String>,
}

/// `GET /api/graph` — the What graph projected to `{nodes, edges, contexts}`,
/// rebuilt from `.product/` on every request (the view is always derived).
pub(crate) async fn graph_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<SessionQuery>,
) -> std::result::Result<axum::Json<ViewGraph>, (axum::http::StatusCode, String)> {
    let session = resolve_session(&state.repo_root, state.session.as_deref(), q.session.as_deref());
    project_graph(&state.repo_root, session.as_ref(), q.product.as_deref())
        .map(axum::Json)
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e))
}

/// `GET /api/pf` — the live `window.PF` projection (§3–§7), rebuilt from
/// `.product/` on every request. The explorer UI at `/` merges this over its
/// demo defaults so it reflects the graph, re-fetching on the `/api/events` SSE.
pub(crate) async fn pf_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<SessionQuery>,
) -> std::result::Result<axum::Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = resolve_session(&state.repo_root, state.session.as_deref(), q.session.as_deref());
    let name = resolve_product_name(&state.repo_root, session.as_ref(), q.product.as_deref())
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e))?;
    load_graph(&state.repo_root, &name)
        .map(|g| axum::Json(super::pf_view::build_pf_view(&g, &state.repo_root, &name)))
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e))
}

/// Load a product's canonical What graph for the view. A known product whose
/// graph hasn't been captured yet resolves to an **empty** graph — the view
/// renders the real (empty) product instead of 404ing, which would leave the
/// explorer silently on its bundled demo data at the start of a session. A
/// graph that exists on disk but fails to load stays an error, so a client
/// keeps its last good fetch rather than flashing an empty view mid-authoring.
fn load_graph(repo_root: &Path, product: &str) -> std::result::Result<DomainGraph, String> {
    let dir = session_dir(repo_root, product);
    match DomainSession::load(&dir) {
        Ok(s) => Ok(s.graph),
        Err(_) if !has_graph(&dir, product)
            && product_core::pf::paths::list_products(repo_root).iter().any(|p| p == product) =>
        {
            Ok(DomainGraph::default())
        }
        Err(_) if !has_graph(&dir, product) => Err(format!("no product '{product}' in this repo")),
        Err(_) => Err(format!("could not load the What graph for product '{product}'")),
    }
}

/// `GET /api/events` — an SSE stream that ticks whenever a `.product/` file
/// changes, so the browser re-fetches `/api/graph`.
pub(crate) async fn events_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Sse<impl tokio_stream::Stream<Item = std::result::Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use tokio_stream::{wrappers::BroadcastStream, StreamExt};

    let stream = BroadcastStream::new(state.changes.subscribe())
        .map(|_| Ok(Event::default().event("changed").data("changed")));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// `GET /api/session` — the workflow session this view follows (its `?session=`,
/// else the server's `--session`, else the active scan), so the view can show
/// which part of the What→How→Build process is in progress.
pub(crate) async fn session_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<SessionQuery>,
) -> axum::Json<serde_json::Value> {
    match resolve_session(&state.repo_root, state.session.as_deref(), q.session.as_deref()) {
        Some((id, s)) => axum::Json(serde_json::json!({ "active": true, "id": &id, "product": &s.product,
            "phase": &s.phase, "until": &s.until, "finalized": s.finalized, "history": &s.history })),
        None => axum::Json(serde_json::json!({ "active": false })),
    }
}

/// The session the view follows: an explicit `?session=` query wins, then the
/// server's `--session` scope, then the most-recently-touched in-progress one.
fn resolve_session(repo_root: &Path, configured: Option<&str>, query: Option<&str>) -> Option<(String, WorkflowSession)> {
    if let Some(id) = query.or(configured) {
        let dir = repo_root.join(".product").join("sessions").join(id);
        return WorkflowSession::load(&dir).ok().map(|s| (id.to_string(), s));
    }
    active_session(repo_root)
}

/// The most-recently-touched session, preferring one still in progress.
fn active_session(repo_root: &Path) -> Option<(String, WorkflowSession)> {
    let dir = repo_root.join(".product").join("sessions");
    let mut found: Vec<(std::time::SystemTime, String, WorkflowSession)> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let session = WorkflowSession::load(&e.path()).ok()?;
            let mtime = std::fs::metadata(workflow_path(&e.path())).and_then(|m| m.modified()).ok()?;
            Some((mtime, e.file_name().to_string_lossy().to_string(), session))
        })
        .collect();
    if found.is_empty() {
        return None;
    }
    found.sort_by_key(|t| t.0);
    let pick = found.iter().rev().find(|(_, _, s)| !s.finalized).or_else(|| found.last())?;
    Some((pick.1.clone(), pick.2.clone()))
}

/// Project the What graph for the client — always the canonical graph (sessions
/// write it directly). A followed `session` only selects which product to
/// render; otherwise the configured product name is used.
fn project_graph(repo_root: &Path, session: Option<&(String, WorkflowSession)>, product_override: Option<&str>) -> std::result::Result<ViewGraph, String> {
    let product = resolve_product_name(repo_root, session, product_override)?;
    let graph = load_graph(repo_root, &product)?;
    let pd = product_core::pf::paths::product_base(repo_root, &product);
    let bp = pd.join("blueprints");
    let bp_dir = if bp.is_dir() { bp } else { pd.join("archetypes") };
    let blueprints = du::blueprint_names(&bp_dir);
    let units = du::load_dir(&pd.join("deployable-units"));
    Ok(to_view_graph_with_how(&graph, &blueprints, &units))
}

/// Resolve the product name: query override → followed session → configured name.
fn resolve_product_name(repo_root: &Path, session: Option<&(String, WorkflowSession)>, product_override: Option<&str>) -> std::result::Result<String, String> {
    let from_session = session.map(|(_, s)| s.product.trim()).filter(|p| !p.is_empty());
    let product = match product_override.map(str::trim).filter(|p| !p.is_empty()).or(from_session) {
        Some(p) => p.to_string(),
        None => ProductConfig::load_from_root(repo_root).map_err(|e| e.to_string())?.name.trim().to_string(),
    };
    if product.is_empty() { return Err("no product configured (set `name` in product.toml)".into()); }
    Ok(product)
}

/// Project the graph following the most-recently-active session (used by tests
/// and the unscoped server default).
#[cfg(test)]
fn load_view(repo_root: &Path) -> std::result::Result<ViewGraph, String> {
    project_graph(repo_root, active_session(repo_root).as_ref(), None)
}

#[cfg(test)]
mod tests {
    use super::load_view;
    use product_core::author::domain::session_dir;
    use product_core::pf::model::{Command, Entity};
    use product_core::pf::session::DomainSession;
    use product_core::pf::viz::{DOMAIN, EVENT};
    use std::path::Path;

    fn save_graph(root: &Path, entities: &[&str]) {
        let mut s = DomainSession::start("demo", None, vec![], None, "t".into()).expect("start");
        for e in entities {
            s.graph.entities.push(Entity {
                id: (*e).into(), label: (*e).into(), context: "ctx".into(), definition: "d".into(),
                ..Default::default()
            });
        }
        s.graph.commands.push(Command { fields: vec![], id: "Place".into(), label: "Place".into(), context: "ctx".into(), targets: "Order".into(), emits: vec![],
        });
        s.save(&session_dir(root, "demo")).expect("save");
    }

    /// view-derivation-verified: the projection mirrors the on-disk What graph,
    /// and a fresh load after a disk change reflects it — proving the view is
    /// rebuilt per request with no cache.
    #[test]
    fn view_graph_reflects_disk_each_call() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");

        save_graph(root, &["Order"]);
        let v1 = load_view(root).expect("load_view");
        assert!(v1.nodes.iter().any(|n| n.id == "Order" && n.model == DOMAIN), "entity in domain lane");
        assert!(v1.nodes.iter().any(|n| n.id == "Place" && n.model == EVENT), "command in event lane");

        save_graph(root, &["Order", "Item"]);
        let v2 = load_view(root).expect("reload");
        assert!(v2.nodes.iter().any(|n| n.id == "Item"), "new node appears without restart (no cache)");
    }

    /// Sessions write the canonical graph directly — the view renders canonical
    /// whether or not a session is in progress (no draft indirection).
    #[test]
    fn view_renders_canonical_during_session() {
        use product_core::author::workflow;
        use product_core::pf::workflow::Phase;

        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");
        save_graph(root, &["Order"]);

        // Scaffold a session (journal only) and write a node to canonical, as
        // the workflow transport now does.
        workflow::scaffold(root, "demo-1", "demo", "claude", Phase::Build, "t".into()).expect("scaffold");
        save_graph(root, &["Order", "Item"]);

        // The view shows the canonical graph, session in progress or not.
        let v = load_view(root).expect("load_view");
        assert!(v.nodes.iter().any(|n| n.id == "Item"), "view renders canonical during a session");
    }

    /// A known product with no captured graph yet (a fresh session, before the
    /// agent's first write) must resolve to an EMPTY graph, not an error — a
    /// 404 left the explorer silently rendering its bundled demo data. An
    /// unknown product stays an error.
    #[test]
    fn fresh_product_loads_as_empty_graph_not_an_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product/products/webshop")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"webshop\"\n").expect("config");

        let g = super::load_graph(root, "webshop").expect("empty graph for a fresh product");
        assert_eq!(g.counts().iter().map(|(_, n)| n).sum::<usize>(), 0, "nothing authored yet");
        assert!(super::load_graph(root, "ghost").is_err(), "unknown product stays an error");
    }

    /// The view follows the *specific* session it is scoped to (or queried),
    /// not merely whichever was touched most recently.
    #[test]
    fn resolve_session_prefers_scope_then_query() {
        use super::resolve_session;
        use product_core::author::workflow;
        use product_core::pf::workflow::Phase;

        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");
        save_graph(root, &["Order"]);

        // Two sessions; "b" is created (touched) after "a".
        workflow::scaffold(root, "a", "demo", "claude", Phase::Build, "t".into()).expect("a");
        workflow::scaffold(root, "b", "demo", "claude", Phase::Build, "t".into()).expect("b");

        // Configured scope wins over the most-recent scan.
        let (id, _) = resolve_session(root, Some("a"), None).expect("scoped");
        assert_eq!(id, "a", "configured --session must win over the recency scan");

        // An explicit query wins over the configured scope.
        let (id, _) = resolve_session(root, Some("a"), Some("b")).expect("queried");
        assert_eq!(id, "b", "?session= must win over the configured scope");

        // Unknown id yields nothing (no silent fallback to the scan).
        assert!(resolve_session(root, Some("ghost"), None).is_none());
    }
}
