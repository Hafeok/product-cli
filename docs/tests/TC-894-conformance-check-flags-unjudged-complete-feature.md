---
id: TC-894
title: conformance check flags unjudged complete feature
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
runner-args: tc_894_conformance_check_flags_unjudged_complete_feature
last-run: 2026-06-10T19:40:39.744653917+00:00
last-run-duration: 0.3s
---

## Scenario — complete feature with a failing verdict violates EXEC-CLOSE-4

**Given** a repository whose only feature is `complete` while its linked TC
holds a `failing` verdict,
**When** the user runs `product conformance check`,
**Then** the process exits 1 and stdout reports `EXEC-CLOSE-4` naming the
TC and its verdict,
**And when** the same TC instead holds the acknowledged `unrunnable`
platform-skip verdict, the check exits 0 — matching `product verify`
completion semantics.

## Validates

- FT-108 — Two Pillars conformance check over the knowledge graph
- ADR-052 — Two Pillars clauses are checked structurally from the knowledge graph