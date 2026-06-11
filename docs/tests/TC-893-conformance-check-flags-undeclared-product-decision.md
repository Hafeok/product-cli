---
id: TC-893
title: conformance check flags undeclared product decision
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
runner-args: tc_893_conformance_check_flags_undeclared_product_decision
last-run: 2026-06-10T19:40:39.744653917+00:00
last-run-duration: 0.3s
---

## Scenario — unanchored feature-specific ADR violates SPEC-DERIVE-3

**Given** an otherwise conforming repository plus an accepted ADR with
default (feature-specific) scope, an empty `features:` list, and no feature
listing it in `adrs:`,
**When** the user runs `product conformance check`,
**Then** the process exits 1 and stdout reports `SPEC-DERIVE-3` against the
ADR's ID — a How element with no What anchor is an undeclared product
decision.

## Validates

- FT-108 — Two Pillars conformance check over the knowledge graph
- ADR-052 — Two Pillars clauses are checked structurally from the knowledge graph