---
id: TC-897
title: author domain serve full session reaches conformant finalize
type: scenario
status: passing
validates:
  features:
  - FT-109
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_897_author_domain_serve_full_session
---

## Scenario — a full served session finalizes conformantly

**Given** the domain MCP server hosted via `product author domain todo
--serve --session-dir <dir>`,
**When** the client sends, over stdin JSON-RPC, `initialize`, `tools/list`,
then `session_start` and the structured operations to build a bounded
context, an entity, an event, a command, and finally `session_finalize`,
**Then** the process exits 0; on stdout, `tools/list` advertises the 17-tool
surface, every `add_*` response carries `ok: true`, and the
`session_finalize` response on stdout carries `ok: true` with the exported
Turtle and a provenance record, and the `<dir>/todo.ttl` and
`<dir>/todo.provenance.json` files are written.

## Validates

- FT-109 — product author domain — facilitated What-capture MCP session
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
