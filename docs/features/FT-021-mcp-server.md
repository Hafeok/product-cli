---
id: FT-021
title: MCP Server
phase: 5
status: complete
depends-on: []
adrs:
- ADR-020
- ADR-031
tests:
- TC-099
- TC-100
- TC-101
- TC-102
- TC-103
- TC-104
- TC-105
- TC-106
- TC-107
- TC-165
domains:
- api
- networking
- security
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

Product exposes its full tool surface as an MCP server. The same binary serves both transports. The transport is a startup flag, not a separate binary.

### Transports

**stdio** — spawned as a subprocess by Claude Code. Standard MCP transport. Local only. No authentication required — the parent process controls access.

```bash
# .mcp.json at repo root — committed, picked up automatically by Claude Code
{
  "mcpServers": {
    "product": {
      "command": "product",
      "args": ["mcp"],
      "cwd": "/path/to/repo"
    }
  }
}
```

**HTTP (Streamable HTTP)** — Product runs as an HTTP server. Any MCP-capable client can connect, including claude.ai configured with a remote MCP server URL. This is the transport for phone access.

```bash
# On your desktop or Pi:
product mcp --http --port 7777 --bind 0.0.0.0 --token $PRODUCT_TOKEN

# In claude.ai Settings → Connectors → Add MCP Server:
# URL:   http://your-machine.local:7777/mcp
# Header: Authorization: Bearer $PRODUCT_TOKEN
```

The HTTP transport implements the MCP Streamable HTTP spec — HTTP POST to `/mcp` for client→server, server-sent events on the same endpoint for streaming responses.

### Tool Surface

MCP tools are a curated subset of the CLI. All tools are read-safe by default. Write tools (scaffold, link, status update) require the `write` capability to be enabled in `product.toml`.

**Read tools (always enabled):**

| Tool | Equivalent CLI |
|---|---|
| `product_context` | `product context FT-XXX --depth N` |
| `product_feature_list` | `product feature list` |
| `product_feature_show` | `product feature show FT-XXX` |
| `product_feature_deps` | `product feature deps FT-XXX` |
| `product_adr_show` | `product adr show ADR-XXX` |
| `product_adr_list` | `product adr list` |
| `product_test_show` | `product test show TC-XXX` |
| `product_graph_check` | `product graph check` |
| `product_graph_central` | `product graph central` |
| `product_impact` | `product impact ADR-XXX` |
| `product_gap_check` | `product gap check ADR-XXX` |
| `product_adr_review` | `product adr review ADR-XXX` |
| `product_metrics_stats` | `product metrics stats` |

**Write tools (require `mcp.write = true` in product.toml):**

| Tool | Equivalent CLI |
|---|---|
| `product_feature_new` | `product feature new "title"` |
| `product_adr_new` | `product adr new "title"` |
| `product_test_new` | `product test new "title" --type TYPE` |
| `product_feature_link` | `product feature link FT-XXX --adr ADR-XXX` |
| `product_adr_status` | `product adr status ADR-XXX accepted` |
| `product_test_status` | `product test status TC-XXX passing` |
| `product_feature_status` | `product feature status FT-XXX complete` |

### Configuration

```toml
# product.toml
[mcp]
write = true              # enable write tools
token = ""                # bearer token for HTTP transport
                          # override with PRODUCT_MCP_TOKEN env var
port = 7777               # default HTTP port
cors-origins = []         # allowed CORS origins for HTTP transport
                          # ["https://claude.ai"] for claude.ai access
```

### Security Model

stdio transport has no authentication — the invoking process owns the repo. HTTP transport requires a bearer token when `--token` is set. Requests without a valid token receive 401. The token is never logged. For remote access from claude.ai, the token is set as a request header in the claude.ai connector configuration.

TLS is not handled by Product. For HTTPS, terminate TLS upstream (nginx, Caddy, Cloudflare Tunnel). Product binds HTTP; the proxy provides TLS.

---

---

## Description

Product exposes its full tool surface as an MCP server accessible over two transports from a single binary. The stdio transport is used by Claude Code (spawned as a subprocess from `.mcp.json`). The HTTP Streamable transport allows any MCP-capable client — including claude.ai on a phone — to connect to a running Product instance over the network. Both transports share the same tool registry, the same graph loading logic, and the same write-safety model (ADR-015 advisory lock). The transport is a startup flag, not a product boundary (ADR-020).

## Functional Specification

### Inputs

- **Transport flag**: `product mcp` (stdio, default) or `product mcp --http [--port N] [--bind ADDR] [--token TOKEN]`
- **Repo path**: resolved from `cwd` or `--repo` flag; `product.toml` is read from this directory
- **MCP JSON-RPC messages**: tool call requests received over stdin (stdio) or HTTP POST to `/mcp` (HTTP)
- **Bearer token**: passed via `--token` flag or `PRODUCT_MCP_TOKEN` env var; required for authenticated HTTP access
- **`product.toml` settings**: `[mcp]` section controls `write`, `token`, `port`, and `cors-origins`

### Outputs

- **stdio**: newline-delimited JSON-RPC responses on stdout; errors on stderr
- **HTTP**: JSON responses to POST `/mcp`; server-sent event streams for long-running tools (e.g. `product_gap_check`)
- **401 Unauthorized**: returned by the HTTP transport when a bearer token is configured and the request carries no valid token
- **Tool error** (not HTTP error): returned when a write tool call cannot acquire the advisory lock within 3 seconds, including the lock-holder's PID

### State

The MCP server is stateless between tool calls. The knowledge graph is rebuilt from YAML front-matter on each tool invocation (ADR-003). No session state is persisted. The HTTP transport is also stateless — concurrent write calls are serialised by the advisory file lock, not by server-side session tracking.

### Behaviour

1. On startup, Product reads `product.toml` and registers all tools in the shared `ToolRegistry`. Write tools are registered only when `mcp.write = true`.
2. The stdio handler reads newline-delimited JSON from stdin and dispatches each message to `ToolRegistry::call`. The HTTP handler accepts POST requests to `/mcp` and dispatches identically.
3. Both transports call the same tool implementations — no logic is duplicated per transport (ADR-020 CLI/MCP parity invariant). Each tool delegates to a shared library function in `src/<slice>/`.
4. Read tools are always available. Write tools require `mcp.write = true` in `product.toml`.
5. On SIGTERM or SIGINT (HTTP mode), the server stops accepting new connections, drains in-flight requests up to 10 seconds, releases any held file lock, and exits 0.
6. TLS is not handled by Product; the operator terminates TLS upstream (Caddy, nginx, Cloudflare Tunnel).

### Invariants

- Every tool surfaced over both `product <cmd>` (CLI) and `product_<cmd>` (MCP) must route through a shared library function in `src/<slice>/`. Neither the CLI adapter nor the MCP handler may contain inline business logic invisible to the other side (ADR-020 parity invariant).
- If a bearer token is configured (`--token` or `PRODUCT_MCP_TOKEN`), every HTTP request without a valid token receives 401. The token is never logged.
- Concurrent write tool calls are serialised by the advisory lock (ADR-015). A write call that cannot acquire the lock within 3 seconds returns a tool error, not an HTTP-level error.
- The tool registry is built once at startup. Tool availability is determined by `product.toml` at launch time, not per-request.

### Error handling

- **Tool not found**: `ToolError::NotFound` — returned as a structured tool error in the MCP response envelope.
- **Write disabled**: `ToolError::WriteDisabled` — returned when a write tool is called without `mcp.write = true`.
- **Lock timeout**: tool error with lock-holder PID; the HTTP connection is not closed.
- **401 Unauthorized**: HTTP-level response when the bearer token is missing or invalid.
- Model or graph errors propagate through the standard `ProductError` model (ADR-013) and are returned as structured tool errors.

### Boundaries

- Product does not implement TLS; TLS termination is the operator's responsibility.
- Product does not implement OAuth; bearer token auth is the maximum authentication complexity for this use case.
- The MCP server does not manage agent sessions or maintain conversational state. Each tool call is independent.
- WebSocket transport is not supported; MCP Streamable HTTP is the HTTP transport standard.
- CORS headers are configurable via `cors-origins` in `product.toml`; they are required for claude.ai browser access.

## Out of scope

- TLS termination (handled by upstream reverse proxy or tunnel)
- OAuth or multi-user authentication (bearer token is sufficient for a personal developer tool)
- Agent session management or conversational context between tool calls
- WebSocket transport
- A separate MCP binary distinct from the main `product` binary
