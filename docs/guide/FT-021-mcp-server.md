It looks like I need write permission to `docs/guide/FT-021-mcp-server.md`. Could you approve the file write? The documentation is ready — it covers all five Diataxis sections:

- **Overview** — what the MCP server is and why it exists
- **Tutorial** — two walkthroughs: Claude Code (stdio) and phone access (HTTP)
- **How-to Guide** — recipes for stdio, HTTP, write tools, CORS, systemd, graceful shutdown
- **Reference** — full CLI syntax, options table, env vars, `product.toml` config, HTTP endpoints, auth model, read/write tool tables, `.mcp.json` format
- **Explanation** — design rationale for dual transport (ADR-020), stdio default, bearer token vs OAuth, TLS delegation, write lock reuse, and CLI-to-MCP tool surface relationship

~230 lines total, all commands and flags verified against `src/main.rs`.
