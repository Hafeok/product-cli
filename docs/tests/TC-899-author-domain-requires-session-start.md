---
id: TC-899
title: author domain tools require session_start first
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
runner-args: tc_899_author_domain_serve_requires_session_start
---

## Scenario — a tool call before session_start is a clear error

**Given** a served session where no `session_start` has been called,
**When** the client calls `add_entity`,
**Then** the process exits 0 and the response on stdout is a JSON-RPC error
whose message tells the caller to call `session_start` first.

## Validates

- FT-109 — product author domain — facilitated What-capture MCP session
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
