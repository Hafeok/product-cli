---
id: TC-1005
title: assisted criterion discharged by attestation
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
runner-args: tc_1005_assisted_criterion_discharged_by_attestation
last-run: 2026-06-20T18:17:51.089850889+00:00
last-run-duration: 0.7s
---

## Scenario — an assisted criterion is discharged only by a dated, attributed attestation

**Given** a captured What graph with a `UiStep` whose obligation union includes a
`WcagCriterion` tagged verification type `assisted` (or `manual`),
**And** no attestation has been recorded for it,
**When** the user runs the accessibility check over the step,
**Then** the process exits non-zero — the criterion is undischarged, because an
assisted/manual criterion cannot be settled by a machine gate.

**And when** a dated, attributed `attests` record is supplied for that criterion,
**then** the criterion is discharged, the check exits 0, and the verdict reports
the step conformant at its stated level with the attestation as the basis for
that criterion.

## Validates

- FT-137 — WCAG accessibility criteria ingestion and attestations
- ADR-081 — Accessibility is ingested WCAG 2.2 criteria with machine, assisted, and manual verification