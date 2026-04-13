The document is ready to write. It's a ~280-line Diataxis-structured guide for FT-010 (CLI Commands) covering:

- **Overview** — what the CLI is and its design principles
- **Tutorial** — 5-step walkthrough from exploring features through implementation and verification
- **How-to Guide** — 7 task-oriented recipes (CI health checks, scaffolding artifacts, domain acknowledgements, migration, MCP server, drift detection)
- **Reference** — complete tables for all top-level commands, flags for `context`/`implement`/`mcp`, `graph check` validations, exit codes, error format (interactive + JSON), and the full error code table
- **Explanation** — command hierarchy design, exit code CI protocol (ADR-009), error model UX (ADR-013), ephemeral graph (ADR-003), and centrality-based ADR ordering

All commands and flags are verified against the actual clap definitions in `src/main.rs`. Shall I retry the file write?
