---
id: TC-895
title: conformance check json reports clauses and profile
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
runner-args: tc_895_conformance_check_json_reports_clauses_and_profile
last-run: 2026-06-10T19:40:39.744653917+00:00
last-run-duration: 0.4s
---

## Scenario — JSON report carries spec id, profile, clauses, findings

**Given** a conforming repository,
**When** the user runs `product conformance check --format json`,
**Then** stdout is a single JSON object with `spec` = `two-pillars/0.1`,
`profile` = `level-3`, zero `summary.violations`, and a `clauses[]` array
containing every registered clause,
**And when** a violating feature is added, the same JSON shape reports
`profile` = `below-level-3` with a non-empty `findings[]` array and the
process exits 1.

## Validates

- FT-108 — Two Pillars conformance check over the knowledge graph
- ADR-052 — Two Pillars clauses are checked structurally from the knowledge graph