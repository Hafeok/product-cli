---
id: TC-898
title: author domain rejects an event that changes a non-entity
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
runner-args: tc_898_author_domain_serve_rejects_invalid_event
---

## Scenario — an invalid event is rejected in-loop

**Given** a served session with a bounded context already created,
**When** the client calls `add_event` for an event whose `changes` target is
not a real entity,
**Then** the process exits 0 and the `add_event` response on stdout carries
`ok: false` with a violation whose message names the framework section `§3.2`
(every event must change a real domain entity); the fragment is not committed.

## Validates

- FT-109 — product author domain — facilitated What-capture MCP session
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
