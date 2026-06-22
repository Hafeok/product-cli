---
id: TC-1029
title: data conformance is adoptable standalone
type: scenario
status: passing
validates:
  features:
  - FT-147
  adrs:
  - ADR-089
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1029_data_conformance_is_adoptable_standalone
last-run: 2026-06-22T19:18:51.924789915+00:00
last-run-duration: 0.4s
---

## Scenario — the data side adopts with none of the rest of the framework

**Given** a captured graph carrying only a domain structure — a context, an
entity, a reference set, and a data-shape — and a bound production dataset, with
no event model, Decider, Projector, UI model, or work units,
**When** the user runs `product domain validate` and then `product domain data`,
**Then** both exit 0: the graph is structurally valid with nothing but its data
side, and data conformance runs end to end, reporting the **data-divergence
rate** over the dataset. Data conformance is the framework's minimal standalone
adoption.

## Validates

- FT-147 — data conformance profile as minimal standalone adoption (§13)
- ADR-089 — domain model structure/data split