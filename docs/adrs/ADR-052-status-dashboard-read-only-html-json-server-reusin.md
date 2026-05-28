---
id: ADR-052
title: Status Dashboard — Read-Only HTML/JSON Server Reusing the Status Slice
status: proposed
features:
- FT-105
supersedes: []
superseded-by: []
domains:
- api
- error-handling
- networking
- observability
- security
scope: feature-specific
---

**Status:** Proposed

**Context:** `product status` already produces a complete project-health
summary from the knowledge graph: features by phase, test coverage,
untested/failing rollups, and per-phase detail. Today that summary is only
reachable from a terminal — a developer who wants to glance at progress
from a phone, a TV in the team room, or a browser tab on the second
monitor has no zero-friction option. The MCP HTTP transport (ADR-020)
already proves that the binary can stand up an axum-based listener,
authenticate with a bearer token, and shut down cleanly under
systemd-style signals; reusing that machinery for a tiny read-only
dashboard is far cheaper than a sidecar tool.

The dashboard must not become a second source of truth for status. Five
parity features (FT-046, FT-059, FT-062, FT-066, FT-069) closed the same
class of bug: a second adapter quietly omitted validation or rendering
the CLI performed. The same trap exists here — an HTML view that
re-implements `status::build_project_status` will diverge inside a week.
The dashboard therefore stays inside the Slice + Adapter discipline
(PAT-001): it is yet another adapter on top of `src/status/`, sharing
the same plan/build/render seam as the CLI.

**Decision:** Add a `product serve` command that mounts an axum router
over the existing status slice. The router exposes a read-only HTML
dashboard and a JSON twin of every screen. There are no write endpoints,
no client-side framework, no persistent server state beyond the graph
loader.

---

### Command surface

```bash
product serve                                   # bind 127.0.0.1:7780, no auth
product serve --port 8080 --bind 0.0.0.0        # LAN access
product serve --token $TOKEN                    # bearer-token gate
product serve --open                            # open default browser
```

`[serve]` block in `.product/config.toml` provides defaults: `port`,
`bind`, `token`, `cors-origins`, `refresh-seconds`. CLI flags override.
The `PRODUCT_SERVE_TOKEN` env var overrides the file token (mirrors
`PRODUCT_MCP_TOKEN`).

### Routes

| Method | Path                  | Returns                                              |
|--------|-----------------------|------------------------------------------------------|
| GET    | `/`                   | HTML dashboard — phase rollup, counts, recent drift  |
| GET    | `/features`           | HTML feature index (filter by phase/status query)    |
| GET    | `/features/{id}`      | HTML feature detail with linked ADRs and TCs         |
| GET    | `/adrs`               | HTML ADR index                                       |
| GET    | `/adrs/{id}`          | HTML ADR detail                                      |
| GET    | `/tests`              | HTML TC index (filter by status)                     |
| GET    | `/api/status.json`    | Identical bytes to `product status --format json`    |
| GET    | `/api/features.json`  | Identical bytes to `product feature list --format json` |
| GET    | `/api/adrs.json`      | Identical bytes to `product adr list --format json`  |
| GET    | `/healthz`            | `200 ok` (no body) — liveness probe                  |
| Other  | any                   | `405 Method Not Allowed` (GET-only server)           |
| GET    | unknown               | `404 Not Found` with rustc-style error envelope      |

### Architecture — adapter only, no new slice

```rust
// src/serve/router.rs — pure plan
pub fn build_router(graph_loader: Arc<dyn GraphLoader>, auth: AuthPolicy) -> Router {
    Router::new()
        .route("/",                get(handlers::dashboard))
        .route("/features",        get(handlers::features_index))
        .route("/features/:id",    get(handlers::feature_detail))
        // ...
        .layer(middleware::from_fn_with_state(auth, auth_layer))
        .layer(middleware::from_fn(method_guard))
}

// src/serve/handlers.rs — every handler is a 5-line wrapper
async fn dashboard(State(s): State<AppState>) -> Response {
    let graph = s.loader.load()?;                          // ADR-003: fresh each request
    let plan  = status::build_project_status(&graph);       // shared with CLI
    render::html::dashboard(&plan).into_response()
}
```

`status::build_project_status`, `feature::list`, `adr::list`,
`test::list` are reused verbatim. The serve adapter only adds:

1. axum routing.
2. Bearer-token middleware (lifted from `mcp::http`).
3. HTML rendering through `askama` templates (already justified by the
   review/onboarding flows that use string templates today — askama is a
   small step up with compile-time checking).
4. JSON rendering via the existing `Serialize` derives.

### Rendering choices

- **Server-rendered HTML, no JavaScript build step.** A single CSS file
  is bundled at compile time via `include_str!`; no asset pipeline.
- **`<meta http-equiv="refresh" content="N">`** for auto-refresh. `N`
  comes from `[serve].refresh-seconds` (default 30). Acceptable because
  the dashboard is a glance-tool, not a control panel.
- **Progressive enhancement only.** Pages function with JS disabled.
- **No cookies, no sessions.** Bearer token is the only state.

### Security model (mirrors ADR-020)

- stdio-equivalent loopback default: `127.0.0.1` with no token is
  acceptable.
- Any non-loopback bind without a token logs a `W***` warning at startup
  and continues — the operator opted in.
- Bearer token via `Authorization: Bearer <token>` header **or**
  `?token=<token>` query string (so a phone bookmark can carry it).
  Query-string tokens are scrubbed from access logs.
- TLS is delegated upstream (Caddy, Cloudflare Tunnel, nginx). The
  serve binary speaks plain HTTP.
- CORS configurable per `cors-origins`. Empty list disables CORS
  entirely.

### Lifecycle

- Boot logs the bound address and prints the dashboard URL on stdout.
- Reads the graph **per request**, not at boot. The graph is small
  (76 features, 50 ADRs, ~720 TCs in this repo today) and FT-016's
  loader is already millisecond-class; persistent in-memory caching
  is YAGNI and an obvious divergence risk versus the CLI.
- SIGTERM / SIGINT triggers axum's `graceful_shutdown` with a 10-second
  drain (identical timeout to ADR-020).

### Error model

Every error renders through the existing `ProductError` chain:

- HTML routes render an error page that shows the `E***`/`W***` code,
  the rustc-style diagnostic block, and a "back" link.
- `/api/*.json` routes return `application/json` with the same envelope
  as `--format json` (errors, warnings, summary).
- Internal errors (Tier 4) return `500` with the `I***` envelope and
  log to stderr exactly as the CLI does.

### Write safety

The dashboard is read-only. `product serve` does not acquire the repo
write lock and does not register any handler that calls `fileops::*`.
This is the single most important invariant — TC-870 asserts it
directly.

### CLI/MCP/Serve parity

Per the parity invariant amended onto ADR-020, every shared piece of
logic must live in `src/<slice>/`. The serve adapter MUST NOT
re-implement status aggregation, filtering, or rendering of structured
data. Any future feature that drifts from this rule reopens the
FT-046/059/062/066/069 series with a new parity invariant TC.

**Rationale:**

- Axum, tokio, serde, and the askama-style templating story are already
  in the dependency tree (or are sub-100KB additions). No new heavy
  dependencies.
- Server-rendered HTML with `meta refresh` is the right complexity
  level for a personal/team dashboard. SPA, websockets, and SSE are
  rejected as overkill for a glance tool.
- Reading the graph per request keeps the dashboard guaranteed-fresh
  and avoids inventing a cache-invalidation scheme. The CLI does the
  same on every invocation; the dashboard should not be more clever.
- Bearer-token + upstream-TLS matches the MCP HTTP transport exactly,
  so operators learn one security story.

**Rejected alternatives:**

- **Embed dashboard in `product mcp --http`.** Rejected: conflates the
  agent protocol (MCP) with a human-facing UI. Different audiences,
  different content-types, different rate-of-change.
- **Single-page app with a `/api/graph` GraphQL endpoint.** Rejected:
  adds a build pipeline (npm, bundler), an asset story, and a parity
  vector for query semantics. Server-rendered HTML solves the stated
  need.
- **Static-site generator that emits HTML to disk.** Rejected: removes
  the freshness guarantee and forces the user to re-run a build step
  after every `product verify`. The whole point is to glance at *now*.
- **Tauri / Electron desktop app.** Way over-budget for a glance tool.
- **Re-implement status aggregation in a new `src/dashboard/` slice.**
  Rejected by the parity invariant. The slice is `src/status/`, full
  stop; serve is an adapter, not a slice.
