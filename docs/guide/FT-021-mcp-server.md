## Overview

Product exposes its entire tool surface as an MCP (Model Context Protocol) server, allowing AI assistants like Claude Code and claude.ai to query and manipulate the knowledge graph directly. A single `product mcp` command serves both stdio transport (for local subprocess spawning) and HTTP Streamable transport (for remote access from phones, browsers, and other MCP clients). The transport is a startup flag, not a separate binary — tool logic has no transport awareness.

## Tutorial

### Connecting Claude Code to your knowledge graph

In this tutorial you will configure Claude Code to use Product as an MCP server so that Claude can query features, ADRs, and test criteria directly.

1. Open your repository root and create (or edit) the `.mcp.json` file:

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

2. Commit the file so every collaborator picks it up automatically:

   ```bash
   git add .mcp.json
   git commit -m "Add Product MCP server config"
   ```

3. Restart Claude Code. It detects `.mcp.json` and spawns `product mcp` as a subprocess. You can now ask Claude to query your knowledge graph — for example, "Show me the context bundle for FT-021" or "List all features."

4. Verify the connection by asking Claude to call `product_feature_list`. You should see the same output as running `product feature list` in your terminal.

### Accessing the knowledge graph from your phone

This tutorial sets up the HTTP transport so you can query your knowledge graph from claude.ai on any device.

1. Generate a bearer token:

   ```bash
   export PRODUCT_MCP_TOKEN=$(openssl rand -hex 32)
   echo "$PRODUCT_MCP_TOKEN"   # save this somewhere safe
   ```

2. Start the HTTP server:

   ```bash
   product mcp --http --bind 0.0.0.0 --port 7777 --token "$PRODUCT_MCP_TOKEN"
   ```

3. If you need HTTPS for remote access (outside your LAN), set up a tunnel:

   ```bash
   cloudflared tunnel --url http://localhost:7777
   ```

   Note the tunnel URL that Cloudflare prints.

4. In claude.ai, go to **Settings → Connectors → Add MCP Server**. Enter:
   - **URL:** `https://your-tunnel.cfargotunnel.com/mcp` (or `http://your-machine.local:7777/mcp` on LAN)
   - **Header:** `Authorization: Bearer <your-token>`

5. Test by asking Claude on your phone to "list all features in the product knowledge graph."

## How-to Guide

### Start the MCP server in stdio mode

1. Run `product mcp` in your repository directory.
2. The server reads `product.toml` from the current working directory and communicates via stdin/stdout using newline-delimited JSON-RPC.

To specify an explicit repo path:

```bash
product mcp --repo /path/to/repo
```

### Start the MCP server in HTTP mode

1. Run:

   ```bash
   product mcp --http --port 7777 --bind 127.0.0.1 --token "$PRODUCT_MCP_TOKEN"
   ```

2. The server listens on the specified address and port. Clients send HTTP POST requests to `/mcp`.

### Enable write tools

By default, MCP tools are read-only. To allow Claude to create features, link artifacts, or update statuses:

1. Edit `product.toml`:

   ```toml
   [mcp]
   write = true
   ```

2. Restart the MCP server.

### Configure CORS for claude.ai browser access

1. Edit `product.toml`:

   ```toml
   [mcp]
   cors-origins = ["https://claude.ai"]
   ```

2. Restart the HTTP server. The server now includes the appropriate CORS headers for preflight and actual requests from claude.ai.

### Run as a systemd service

1. Create a service file that runs `product mcp --http` with your desired flags.
2. The server handles SIGTERM gracefully: it stops accepting new connections, drains in-flight requests (up to 10 seconds), releases file locks, and exits cleanly.

### Set the token via environment variable

Instead of passing `--token` on the command line, set the `PRODUCT_MCP_TOKEN` environment variable:

```bash
export PRODUCT_MCP_TOKEN="your-secret-token"
product mcp --http --bind 0.0.0.0 --port 7777
```

The environment variable takes effect if `--token` is not explicitly provided. You can also set the token in `product.toml`:

```toml
[mcp]
token = "your-secret-token"
```

The `PRODUCT_MCP_TOKEN` environment variable overrides the `product.toml` value.

## Reference

### CLI syntax

```
product mcp [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--http` | off (stdio) | Switch to HTTP Streamable transport |
| `--port <PORT>` | `7777` | HTTP listen port |
| `--bind <ADDR>` | `127.0.0.1` | HTTP bind address (`0.0.0.0` for remote access) |
| `--token <TOKEN>` | none | Bearer token for HTTP authentication |
| `--repo <PATH>` | current directory | Path to the repository root |

### Configuration keys (`product.toml`)

```toml
[mcp]
write = true              # Enable write tools (default: false)
token = ""                # Bearer token for HTTP transport
port = 7777               # Default HTTP port
cors-origins = []         # Allowed CORS origins, e.g. ["https://claude.ai"]
```

### Environment variables

| Variable | Overrides | Description |
|----------|-----------|-------------|
| `PRODUCT_MCP_TOKEN` | `[mcp] token` and `--token` | Bearer token for HTTP authentication |

### HTTP endpoint

| Method | Path | Description |
|--------|------|-------------|
| POST | `/mcp` | Client-to-server MCP requests. Returns inline JSON or server-sent events for streaming responses. |

### Authentication (HTTP only)

Requests must include `Authorization: Bearer <token>` when a token is configured. Missing or invalid tokens receive HTTP `401 Unauthorized`. If no token is configured, the server starts with a warning — acceptable for `--bind 127.0.0.1`, not recommended for `--bind 0.0.0.0`.

### Read tools (always enabled)

| MCP Tool Name | Equivalent CLI | Description |
|---------------|---------------|-------------|
| `product_context` | `product context FT-XXX --depth N` | Assemble context bundle |
| `product_feature_list` | `product feature list` | List all features |
| `product_feature_show` | `product feature show FT-XXX` | Show feature details |
| `product_feature_deps` | `product feature deps FT-XXX` | Show feature dependencies |
| `product_adr_show` | `product adr show ADR-XXX` | Show ADR details |
| `product_adr_list` | `product adr list` | List all ADRs |
| `product_test_show` | `product test show TC-XXX` | Show test criterion |
| `product_graph_check` | `product graph check` | Check graph health |
| `product_graph_central` | `product graph central` | Show centrality metrics |
| `product_impact` | `product impact ADR-XXX` | Show ADR impact analysis |
| `product_gap_check` | `product gap check ADR-XXX` | Run gap analysis |
| `product_adr_review` | `product adr review ADR-XXX` | Review an ADR |
| `product_metrics_stats` | `product metrics stats` | Show architecture metrics |

### Write tools (require `mcp.write = true`)

| MCP Tool Name | Equivalent CLI | Description |
|---------------|---------------|-------------|
| `product_feature_new` | `product feature new "title"` | Create a feature |
| `product_adr_new` | `product adr new "title"` | Create an ADR |
| `product_test_new` | `product test new "title" --type TYPE` | Create a test criterion |
| `product_feature_link` | `product feature link FT-XXX --adr ADR-XXX` | Link feature to ADR |
| `product_adr_status` | `product adr status ADR-XXX accepted` | Update ADR status |
| `product_test_status` | `product test status TC-XXX passing` | Update test status |
| `product_feature_status` | `product feature status FT-XXX complete` | Update feature status |

### Write tool errors

When a write tool is called but `mcp.write = false`, the server returns a **tool error** (not an HTTP error) with the message `"write tools disabled"`.

When concurrent write calls contend for the advisory file lock, the call that cannot acquire the lock within 3 seconds returns a tool error containing the lock-holder's PID.

### Graceful shutdown (HTTP mode)

On SIGTERM or SIGINT the server:

1. Stops accepting new connections.
2. Completes in-flight requests (up to 10-second drain timeout).
3. Releases any held file lock.
4. Exits with code 0.

## Explanation

### Why a single binary with dual transport?

Two separate binaries (`product-mcp-stdio` and `product-mcp-http`) would inevitably diverge on tool surface, error handling, and graph loading. The transport is a thin wire-protocol layer; the tool logic is shared. A single `product mcp` command with `--http` keeps both transports in lockstep with zero duplication. See ADR-020 for the full rationale.

### Why MCP Streamable HTTP instead of WebSocket or gRPC?

MCP Streamable HTTP is the current MCP specification for remote servers and has the broadest client support, including claude.ai. WebSocket is supported by some clients but is being superseded. gRPC is excellent for high-throughput service-to-service communication but is overkill for a developer tool handling tens of requests per session. See ADR-020.

### Why bearer token instead of OAuth?

Product is a personal developer tool, not a multi-user SaaS platform. A static bearer token stored in a password manager or environment variable is the right complexity level. OAuth would add authorization server infrastructure and token lifecycle management for no practical benefit in this context.

### Why delegate TLS to a reverse proxy?

Implementing TLS directly would add a dependency (rustls or openssl), certificate management, and renewal complexity. Cloudflare Tunnel, Caddy, or nginx handle TLS termination reliably and provide a publicly accessible HTTPS endpoint with minimal configuration. Product binds plain HTTP; the proxy provides HTTPS to clients.

### Write safety and concurrency

HTTP transport is stateless — multiple clients could send concurrent write requests. Product reuses the same advisory file lock (ADR-015) that serializes concurrent CLI invocations. This means MCP write calls and CLI write commands are safely serialized, preventing corruption regardless of which interface initiates the write.

### Relationship to the knowledge graph

The MCP server does not maintain a persistent graph. Like the CLI, it rebuilds the graph from YAML front-matter on every tool invocation (ADR-003). This guarantees that MCP responses always reflect the current state of the repository, with no cache invalidation concerns.
