---
id: TC-1003
title: step inherits accessibility obligations from its AIOs
type: scenario
status: unimplemented
validates:
  features:
  - FT-137
  adrs:
  - ADR-081
phase: 7
observes:
- graph
- stdout
runner: cargo-test
runner-args: tc_1003_step_inherits_accessibility_obligations_from_its_aios
---

## Scenario — a step's accessibility obligation is the computed union of its AIOs' criteria

**Given** a captured What graph with the seed AIO set, where `text-entry`
declares its labelling criteria via `must_satisfy` and an image-bearing
`display-value` declares 1.1.1 Non-text Content,
**And** a `UiStep` `EditProfile` that references a `text-entry` AIO,
**When** the user runs `product domain show EditProfile`,
**Then** the process exits 0 and stdout reports the step's accessibility
obligation as the **computed union** of its AIOs' criteria — the labelling
criteria appear, each annotated with the AIO it is inherited from, and no
hand-maintained per-screen list is involved.

**And when** an image-bearing `display-value` AIO is added to the step, **then**
1.1.1 Non-text Content appears in the union; **and when** the `text-entry` AIO
is removed, **then** its labelling criteria drop out of the union automatically.

## Validates

- FT-137 — WCAG accessibility criteria ingestion and attestations
- ADR-081 — Accessibility is ingested WCAG 2.2 criteria with machine, assisted, and manual verification
