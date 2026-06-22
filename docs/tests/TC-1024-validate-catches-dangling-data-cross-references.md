---
id: TC-1024
title: validate catches dangling data cross references
type: scenario
status: passing
validates:
  features:
  - FT-145
  adrs:
  - ADR-089
phase: 7
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_1024_validate_catches_dangling_data_cross_references
---

## Scenario — a data shape's pointer must resolve to a real concept

**Given** a captured What with an entity `Order`,
**And** a `data-shape` authored to target a non-existent entity,
**When** the user runs `domain validate`,
**Then** the process exits non-zero,
**And** the report names the offending shape — the data-side cross-references
(reference-set concept, shape target, dataset shape) are checked just like the
structure side's, so a shape cannot point at nothing.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
