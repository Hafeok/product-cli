---
id: ADR-020
title: MCP Server — Dual Transport (stdio and HTTP)
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Product must be usable from two distinct environments with fundamentally different connectivity models:

1. **Local desktop** — Claude Code runs as a subprocess in the same OS session as the developer. The natural MCP transport here is stdio: Claude Code spawns `product mcp` as a child process and communicates over stdin/stdout. No network, no authentication, no configuration beyond `.mcp.json`.

2. **Remote client (phone, browser, remote agent)** — claude.ai on a phone cannot spawn subprocesses. It connects to MCP servers over HTTP via the MCP Streamable HTTP transport. Product must bind to a network port, accept HTTP requests, and authenticate them.

Both use cases share the same tool surface. The transport is not a product boundary — it is a wire protocol. Implementing two separate binaries, or two separate tool registrations, would create maintenance burden and inevitable divergence. A single `product mcp` command with a transport flag is the correct design.

**Decision:** `product mcp` defaults to stdio transport. `product mcp --http` switches to HTTP Streamable transport. The tool registry, graph loading, and all tool handlers are shared between transports. Authentication is a transport-layer concern: stdio has none (trust the parent process), HTTP requires a bearer token.

---

### stdio Transport

```bash
product mcp           # stdio, reads repo from cwd
product mcp --repo /path/to/repo   # explicit repo path
```

Wire protocol: newline-delimited JSON over stdin/stdout per the MCP spec. Claude Code spawns this as a subprocess. The `.mcp.json` at repo root is the configuration contract.

```json
{
  "mcpServers": {
    "product": {
      "command": "product",
      "args": ["mcp"],
      "cwd": "${workspaceFolder}"
    }
  }
}
```

`${workspaceFolder}` is resolved by Claude Code to the open repository root. Product reads `product.toml` from this directory.

---

### HTTP Transport (Streamable HTTP)

```bash
product mcp --http
product mcp --http --port 8080
product mcp --http --bind 127.0.0.1    # localhost only
product mcp --http --bind 0.0.0.0      # all interfaces (remote access)
product mcp --http --token $SECRET
```

**Protocol:** MCP Streamable HTTP. Client sends HTTP POST to `/mcp`. Server responds either inline (for non-streaming tools) or as a server-sent event stream (for long-running tools like `product_gap_check`). A single endpoint handles both.

**Authentication:** Bearer token in the `Authorization` header. If `--token` is set (or `PRODUCT_MCP_TOKEN` env var), all requests without a valid token receive `401 Unauthorized`. If no token is configured, the server starts but logs a warning — unauthenticated HTTP is acceptable for localhost-only (`--bind 127.0.0.1`) but not for remote access.

**TLS:** Not handled by Product. The operator terminates TLS upstream. Recommended setups:
- **Local network:** HTTP is acceptable — traffic stays on the LAN
- **Remote access:** Cloudflare Tunnel, ngrok, or a reverse proxy (Caddy, nginx) provides TLS termination. Product binds HTTP; the tunnel provides HTTPS to the client.

**CORS:** Configurable in `product.toml`. For claude.ai access: `cors-origins = ["https://claude.ai"]`.

**Phone setup (complete):**
```bash
# On desktop/server:
export PRODUCT_MCP_TOKEN=$(openssl rand -hex 32)
product mcp --http --bind 0.0.0.0 --port 7777

# Or with Cloudflare Tunnel for HTTPS:
cloudflared tunnel --url http://localhost:7777

# In claude.ai → Settings → Connectors → Add MCP Server:
# URL:    https://your-tunnel.cfargotunnel.com/mcp
# Header: Authorization: Bearer $PRODUCT_MCP_TOKEN
```

---

### Tool Registry

Tools are registered once. The transport layer calls them identically:

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    write_enabled: bool,
}

impl ToolRegistry {
    pub async fn call(&self, name: &str, args: Value) -> ToolResult {
        let tool = self.tools.get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        if tool.requires_write() && !self.write_enabled {
            return Err(ToolError::WriteDisabled);
        }
        tool.call(args).await
    }
}
```

The stdio handler and the HTTP handler both call `ToolRegistry::call`. There is no code path that is transport-specific in tool implementation.

---

### Write Safety in HTTP Mode

HTTP transport is stateless — multiple clients could theoretically send concurrent write requests. The same advisory lock (ADR-015) that serialises concurrent CLI invocations also serialises concurrent MCP write calls. A write tool call that cannot acquire the lock within 3 seconds returns a tool error (not an HTTP error) with the lock-holder's PID.

---

### Graceful Shutdown

HTTP mode responds to SIGTERM and SIGINT. On signal:
1. Stop accepting new connections
2. Complete in-flight requests (up to 10 second drain timeout)
3. Release file lock if held
4. Exit 0

This ensures that a `product mcp --http` process running as a systemd service restarts cleanly.

---

**Rationale:**
- Single binary, dual transport is the correct design. Two binaries would diverge on tool surface, error handling, and graph loading. The transport is genuinely a thin layer — the tool logic has no transport awareness.
- MCP Streamable HTTP is the current MCP specification for remote servers. SSE-based (the older spec) is also supported by claude.ai but is being superseded. Implementing Streamable HTTP positions Product correctly for the current and future spec.
- Bearer token auth is sufficient for this use case. OAuth would be more appropriate for a multi-user SaaS tool. Product is a personal developer tool — a static bearer token stored in a password manager or environment variable is the right complexity level.
- TLS delegation to a reverse proxy is standard practice for application servers written in Rust. Implementing TLS in Product would add a dependency (rustls or openssl), a certificate management problem, and certificate renewal complexity. Cloudflare Tunnel eliminates all of this and provides a publicly accessible HTTPS endpoint in one command.
- CORS is required for claude.ai access from a browser — the browser enforces CORS policy before any MCP request reaches the server. Configuring `cors-origins = ["https://claude.ai"]` in `product.toml` is the minimal configuration for phone access.

**Rejected alternatives:**
- **Two separate binaries: `product-mcp-stdio` and `product-mcp-http`** — maintenance burden, inevitable divergence. Rejected.
- **WebSocket transport** — supported by some MCP clients but not the primary transport for claude.ai. Streamable HTTP has broader client support and simpler server implementation.
- **gRPC** — excellent for high-throughput service-to-service communication. Overkill for a developer tool handling tens of requests per session.
- **Product-as-daemon with IPC** — one `product` daemon, CLI and MCP both talk to it via a Unix socket. Eliminates the cold-start cost of graph loading per invocation. Rejected for v1: the daemon lifecycle (start, stop, version skew between daemon and CLI) adds operational complexity that is not justified at the current scale.