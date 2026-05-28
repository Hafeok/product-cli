---
id: FT-105
title: Status Dashboard Web Server
phase: 6
status: planned
depends-on:
- FT-014
- FT-021
adrs:
- ADR-052
- ADR-020
- ADR-013
- ADR-043
- ADR-047
- ADR-048
- ADR-050
- ADR-051
- ADR-018
- ADR-040
- ADR-042
- ADR-041
- ADR-049
tests:
- TC-864
- TC-865
- TC-866
- TC-867
- TC-868
- TC-869
- TC-870
- TC-871
- TC-872
- TC-873
- TC-874
- TC-875
domains:
- api
- error-handling
- networking
- observability
- security
domains-acknowledged: {}
patterns:
- PAT-001
---

## Description

`product serve` starts a lightweight read-only HTTP server that renders
the project status — phases, features, ADRs, and test criteria — as a
browsable HTML dashboard with parallel JSON endpoints. The dashboard
is a thin adapter on top of the existing `src/status/` slice and reuses
the axum + bearer-token plumbing introduced by the MCP HTTP transport
(ADR-020). It exists so that a developer (or a team member) can glance
at project health from a phone, a wall-mounted display, or a second
monitor without opening a terminal.

The dashboard is deliberately small: server-rendered HTML, no client
framework, no build pipeline, no write surface, no persistent state
beyond the per-request graph load. ADR-052 captures the architecture,
the parity constraint, and every rejected alternative.

## Functional Specification

### Inputs

**CLI flags (parsed by clap on `product serve`):**

| Flag                 | Type     | Default               | Notes                                       |
|----------------------|----------|-----------------------|---------------------------------------------|
| `--port`             | u16      | `7780`                | Override `[serve].port`.                    |
| `--bind`             | string   | `127.0.0.1`           | IPv4 / IPv6 listen address.                 |
| `--token`            | string   | unset                 | Bearer-token gate. Overrides config + env.  |
| `--refresh-seconds`  | u32      | `30`                  | `<meta http-equiv="refresh">` interval.     |
| `--open`             | bool     | `false`               | Best-effort open default browser at boot.   |
| `--no-cors`          | bool     | `false`               | Force-disable CORS regardless of config.    |

**Config block (`.product/config.toml`):**

```toml
[serve]
port = 7780
bind = "127.0.0.1"
token = ""                  # PRODUCT_SERVE_TOKEN env var wins
cors-origins = []           # e.g. ["https://dashboard.local"]
refresh-seconds = 30
```

**Per-request inputs:** HTTP method, path, optional `Authorization`
header, optional `?token=` query parameter, query filters
(`?phase=N`, `?status=planned`, etc.) on list pages.

**Repository inputs:** every `docs/features/*.md`, `docs/adrs/*.md`,
`docs/tests/*.md`, `docs/deps/*.md` reachable from the configured
paths. Loaded fresh per request via `shared::load_graph_typed`.

### Outputs

- **HTML pages** (`text/html; charset=utf-8`) for `/`, `/features`,
  `/features/{id}`, `/adrs`, `/adrs/{id}`, `/tests`.
- **JSON envelopes** (`application/json`) for `/api/status.json`,
  `/api/features.json`, `/api/adrs.json`. Byte-for-byte identical to
  the CLI `--format json` output for the corresponding subcommand
  (parity invariant TC-868).
- **`200 ok\n`** plain-text for `/healthz`.
- **Error responses** rendered through `ProductError`:
  - HTML routes → styled error page carrying the `E***` / `W***` code.
  - JSON routes → `{ "errors": [...], "warnings": [...], "summary": {...} }`
    matching ADR-013.
- **stdout at boot** prints `serving dashboard on http://<bind>:<port>/`
  and (when `--open` succeeds) `opening browser…`.
- **stderr** carries Tier-4 internal errors and startup warnings (e.g.
  non-loopback bind without a token).
- **Exit codes**: `0` on graceful shutdown, `1` on bind failure
  (ProductError::Io), `3` on internal error.

### State

- **In-process only.** `AppState { loader: Arc<dyn GraphLoader>, auth: AuthPolicy }`.
- **No on-disk state.** The serve command never opens the write lock,
  never calls `fileops::write_*`, and never mutates `.product/`.
- **Graph cache:** none. Every request rebuilds the graph from disk.
  This is the same cost the CLI pays on every invocation (ADR-003) and
  guarantees the dashboard is never stale.
- **Connection state:** axum's default — no sessions, no cookies.

### Behaviour

1. **Boot.** `product serve` parses flags, merges `[serve]` config,
   resolves the bind address, configures the auth policy, and binds
   the listener. On success it prints the dashboard URL and (with
   `--open`) launches the default browser. On bind failure it returns
   `ProductError::Io` and exits `1`.
2. **Per request.**
   1. Middleware enforces `GET`-only (anything else → `405`).
   2. Auth middleware checks the bearer token if one is configured.
      Missing / wrong token → `401` (HTML or JSON depending on route).
   3. The handler loads the graph, calls the appropriate `status::`,
      `feature::`, `adr::`, or `tc::` shared function, and renders the
      result through `serve::render::html` or `serve::render::json`.
   4. Errors propagate as `ProductError` and are rendered by the
      transport's error layer.
3. **Auto-refresh.** HTML pages include
   `<meta http-equiv="refresh" content="{refresh_seconds}">` when
   `refresh_seconds > 0`. Setting it to `0` disables refresh.
4. **Shutdown.** SIGTERM / SIGINT triggers axum's graceful shutdown
   with a 10-second drain. The process exits `0` once in-flight
   requests complete.

### Invariants

- **No writes.** No handler in `src/serve/` may call `fileops::write_*`,
  acquire `ProductLock::write`, or otherwise mutate the repository
  (TC-870, asserted via disk-state observation).
- **CLI/MCP/Serve parity.** Status, feature-list, adr-list, and tc-list
  rendering MUST delegate to the shared `src/<slice>/build_*` /
  `render_*` functions. JSON endpoints are byte-equal to their CLI
  counterparts (TC-868).
- **GET-only.** Any non-GET method on any route returns `405` (TC-869).
- **Read-only freshness.** The graph is reloaded on every request; two
  successive requests across an artifact edit see distinct content
  (TC-874).
- **Auth uniformity.** When a token is configured, every route except
  `/healthz` requires it (TC-871).

### Error handling

- `404` for unknown routes — renders an HTML page on browser routes and
  a JSON envelope on `/api/*` routes. JSON shape matches ADR-013
  exactly (TC-872).
- `405` for non-GET methods on any known route.
- `401` for missing / wrong bearer when a token is configured.
- `500 internal error` rendered through ProductError Tier 4 — message
  carries the `I***` code and stderr logs the file:line per ADR-013.
- Bind failure (port in use, permission denied) exits `1` with a
  rustc-style error block on stderr; no listener is opened.
- Graph load failure on a request renders the parse / link error
  envelope (E001/E002/E005/E006) on the page instead of crashing.

### Boundaries

- **In scope:** `product serve` command, read-only HTML dashboard,
  parallel JSON endpoints, bearer-token auth, graceful shutdown,
  `/healthz` liveness probe, query filters for index routes.
- **Out of scope:** see "Out of scope" below.

## Tutorial-level walkthrough

```bash
# Local glance — terminal #1
$ product serve
serving dashboard on http://127.0.0.1:7780/
^C
shutting down (drain ≤ 10s)…

# LAN access with auth
$ export PRODUCT_SERVE_TOKEN=$(openssl rand -hex 32)
$ product serve --bind 0.0.0.0 --port 8080
serving dashboard on http://0.0.0.0:8080/

# Phone bookmark
http://laptop.local:8080/?token=<token>

# CI / scripts
$ curl -s http://127.0.0.1:7780/api/status.json | jq .summary
{
  "phases": 6,
  "features": 76,
  "complete": 76,
  "tests-passing": 713
}
```

## Out of scope

- **Any write operation.** No artifact creation, status change, link,
  amendment, deletion, or migration. Use the CLI or MCP.
- **Authentication beyond bearer token.** No OAuth, no SSO, no per-user
  accounts. The dashboard is a personal/team tool.
- **TLS termination.** Operators terminate TLS upstream (Caddy,
  Cloudflare Tunnel, nginx) — same posture as ADR-020.
- **Real-time updates.** No websockets, no SSE, no long-poll. Auto-refresh
  via `<meta>` is the agreed surface area.
- **Custom themes / asset pipeline.** A single compiled-in CSS file.
  Forks may swap it at build time; runtime theming is out.
- **Multi-repo aggregation.** One repo per `product serve` instance.
- **MCP tool exposure.** `product serve` is not surfaced as an MCP tool;
  it is a long-running process, not a request/response operation.
- **Persistent caching.** Per-request graph load is the contract.
