---
id: TC-1025
title: data shape datatype constraint catches type drift
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
- stdout
runner: cargo-test
runner-args: tc_1025_data_shape_datatype_constraint_catches_type_drift
---

## Scenario — a field declared `integer` rejects a string value

**Given** an `OrderShape` whose `total` field carries a `type` constraint of
`integer`,
**And** a `production-dataset` whose source holds one record with an integer
`total` and one with the string `"twelve"`,
**When** the user runs `domain data OrdersLive`,
**Then** the process exits non-zero,
**And** the verdict names the `not-of-type` defect and reports a 50.0%
divergence rate — datatype is the third checkable shape constraint alongside
required-presence and reference-set membership.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
