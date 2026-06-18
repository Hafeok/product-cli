---
id: TC-958
title: slice new and context assemble the subgraph
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
- stdout
runner: cargo-test
runner-args: tc_958_slice_new_and_context_assemble_the_subgraph
---

## Scenario — a slice points at the model and assembles its context

**Given** a captured What graph (Sales / Order / OrderPlaced / PlaceOrder),
**When** the user runs `product slice new order-slice --anchor Order` and then
`product slice context order-slice`,
**Then** `new` exits 0 and writes `.product/slices/order-slice.yaml`, and
`context` exits 0 emitting a `Domain Context Bundle` that contains the reachable
`PlaceOrder` and `OrderPlaced` — assembled from the graph, not restated. `show`
and `list` surface the slice.

## Validates

- FT-124 — product slice — a saved pointer to a section of the event model
- ADR-065 — A delivery slice is a pointer into the event model; its context is derived from the graph
