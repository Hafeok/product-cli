---
id: TC-1004
title: machine criterion is a deterministic gate
type: scenario
status: passing
validates:
  features:
  - FT-137
  adrs:
  - ADR-081
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1004_machine_criterion_is_a_deterministic_gate
last-run: 2026-06-20T18:17:51.089850889+00:00
last-run-duration: 0.7s
---

## Scenario — an unsatisfied machine criterion fails the gate with a level and a basis

**Given** a captured What graph with a `UiStep` whose obligation union includes a
`WcagCriterion` tagged verification type `machine` (a deterministic gate),
**And** the criterion is unsatisfied,
**When** the user runs the accessibility check over the step,
**Then** the process exits non-zero — a machine criterion is a deterministic
gate, so the failure is mechanical, not a judgement call — and the verdict
reports the conformance **level and its basis** (the criterion, its level, and
that it failed a machine gate), never a bare pass.

**And when** the criterion is satisfied, **then** the gate passes and the verdict
records it as discharged by machine at its stated level.

## Validates

- FT-137 — WCAG accessibility criteria ingestion and attestations
- ADR-081 — Accessibility is ingested WCAG 2.2 criteria with machine, assisted, and manual verification