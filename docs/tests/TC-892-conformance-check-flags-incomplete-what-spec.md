---
id: TC-892
title: conformance check flags incomplete what specification
type: scenario
status: passing
validates:
  features:
  - FT-108
  adrs:
  - ADR-052
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_892_conformance_check_flags_incomplete_what_specification
last-run: 2026-06-10T19:40:39.744653917+00:00
last-run-duration: 0.3s
---

## Scenario — incomplete What unit violates SPEC-WHAT-5 / SPEC-WHAT-8

**Given** an otherwise conforming repository plus a planned feature whose
body has no `Out of scope` section and which links no test criterion,
**When** the user runs `product conformance check`,
**Then** the process exits 1 and stdout names both `SPEC-WHAT-5` and
`SPEC-WHAT-8` against the feature's ID, with the verdict line
`does not conform to Level 3`.

## Validates

- FT-108 — Two Pillars conformance check over the knowledge graph
- ADR-052 — Two Pillars clauses are checked structurally from the knowledge graph