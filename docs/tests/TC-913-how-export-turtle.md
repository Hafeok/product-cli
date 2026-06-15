---
id: TC-913
title: how export emits turtle with synthesised links
type: scenario
status: passing
validates:
  features:
  - FT-111
  adrs:
  - ADR-054
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_913_how_export_emits_turtle_with_synthesised_links
---

## Scenario — project the How contract to Turtle

**Given** the bundled example How contract,
**When** the user runs `product how export`,
**Then** the process exits 0 and stdout is Turtle containing the `@prefix pf:`
header, a `pf:TopDecision`, a synthesised `pf:Verification` (from
`enforced_by`), and a synthesised `pf:WorkUnit` (from `applied_by`).

## Validates

- FT-111 — product how — validate, show, and project an archetype's How contract
- ADR-054 — How contracts are file-based YAML with a native checker mirroring how.shacl.ttl
