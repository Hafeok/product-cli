---
id: TC-1022
title: clean production data has zero divergence
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
runner-args: tc_1022_clean_production_data_has_zero_divergence
---

## Scenario — data that conforms passes with a 0.0% divergence rate

**Given** an authored `OrderShape` (required `id`/`total`, `shipping` constrained
to the `ShippingMethods` reference set) and a `production-dataset` whose source
holds two records that satisfy every constraint,
**When** the user runs `domain data OrdersLive`,
**Then** the process exits zero,
**And** the verdict reports a divergence rate of 0.0% and that all records
conform to the shape — the model earns trust by real data passing its shapes.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
