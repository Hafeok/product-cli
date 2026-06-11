---
id: TC-891
title: conformance check passes on conforming repo
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
runner-args: tc_891_conformance_check_passes_on_conforming_repo
last-run: 2026-06-10T19:40:39.744653917+00:00
last-run-duration: 50.2s
---

## Scenario — conformance check passes on a conforming repository

**Given** a repository with a declared `name`, a `[product].responsibility`,
one feature carrying a full What body (Behaviour, Error handling, Out of
scope), a passing acceptance TC, and an accepted ADR anchored to the
feature,
**When** the user runs `product conformance check`,
**Then** the process exits 0 and stdout contains the clause table with no
`[FAIL]` rows and the verdict line `conforms to Level 3`.

## Validates

- FT-108 — Two Pillars conformance check over the knowledge graph
- ADR-052 — Two Pillars clauses are checked structurally from the knowledge graph