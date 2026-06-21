---
id: TC-1021
title: author the structure data split
type: scenario
status: passing
validates:
  features:
  - FT-145
  adrs:
  - ADR-089
phase: 7
observes:
- graph
- exit-code
- stdout
runner: cargo-test
runner-args: tc_1021_author_the_structure_data_split
---

## Scenario — the data side of §3.1 is first-class in the graph

**Given** a captured What with an entity `Order` in a bounded context,
**When** the user authors a `reference-set` (its constitutive shipping methods),
a `data-shape` over `Order`, and a `production-dataset` pointing at a source,
**Then** `domain validate` reports the graph conformant,
**And** `domain list reference-set` shows the reference data,
**And** `domain export` emits the data-side predicates `pf:referenceDataFor`
and `pf:conformsToShape` — reference data and the production oracle are modelled,
not asserted top-down.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
