---
id: TC-959
title: slice new rejects a dangling anchor
type: scenario
status: passing
validates:
  features:
  - FT-124
  adrs:
  - ADR-065
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_959_slice_new_rejects_a_dangling_anchor
---

## Scenario — a pointer to a non-existent node is refused

**Given** a captured What graph that has no node `Ghost`,
**When** the user runs `product slice new bad --anchor Ghost`,
**Then** the process exits 1, stderr names `Ghost` as not a node in the What
graph, and no `.product/slices/bad.yaml` is written.

## Validates

- FT-124 — product slice — a saved pointer to a section of the event model
- ADR-065 — A delivery slice is a pointer into the event model; its context is derived from the graph
