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
domains-acknowledged: {}
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