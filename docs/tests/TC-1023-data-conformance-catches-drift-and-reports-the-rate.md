---
id: TC-1023
title: data conformance catches drift and reports the rate
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
runner-args: tc_1023_data_conformance_catches_drift_and_reports_the_rate
---

## Scenario — divergence is caught per record and reads both ways

**Given** the `OrderShape` and a `production-dataset` whose source holds three
records, one of which drops the required `total` and carries a `shipping` value
(`drone`) the reference set never declared,
**When** the user runs `domain data OrdersLive`,
**Then** the process exits non-zero,
**And** the verdict reports a 33.3% divergence rate,
**And** it names both defects — `missing-required` and `not-in-reference-set` —
**And** the message reads both ways: fix the data, or (if the spec is stale) fix
the shape. Data conformance is the one check whose failure can indict the
specification.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
