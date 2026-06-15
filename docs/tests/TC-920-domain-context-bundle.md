---
id: TC-920
title: domain context emits bundle with focus and neighbours
type: scenario
status: passing
validates:
  features:
  - FT-112
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_920_domain_context_emits_bundle_with_focus_and_neighbours
---

## Scenario — assemble a bundle around a focus node

**Given** a captured What graph with a Sales context, an Order entity, an
OrderPlaced event, and a PlaceOrder command,
**When** the user runs `product domain context Order --depth 1`,
**Then** the process exits 0 and stdout is a markdown bundle whose header names
Order, whose summary block records `focus≜Order:Entity`, that includes
Order's definition, and that lists its direct neighbours (the OrderPlaced
event and the PlaceOrder command) under their kind sections.

## Validates

- FT-112 — product domain context — assemble an LLM context bundle from the What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
