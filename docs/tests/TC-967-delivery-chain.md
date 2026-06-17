---
id: TC-967
title: delivery chain release feature slice
type: scenario
status: passing
validates:
  features:
  - FT-126
  adrs:
  - ADR-067
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_967_delivery_chain_release_feature_slice
---

## Scenario — the full delivery chain over the event model

**Given** a captured What graph and a slice `order-slice` anchored on `Order`,
**When** the user runs `product deliverable new place-order --slice order-slice
--accept "a1:an order can be placed"` and then `product release new R1 --feature
place-order`,
**Then** both exit 0 and write their pointer files; `deliverable show` reports
`slice: order-slice`, `release show` lists `place-order`, and `product status`
shows `1 slices, 1 deliverables, 1 releases`.

## Validates

- FT-126 — product deliverable and release — the delivery layer over slices
- ADR-067 — Delivery units are id-pointers — release to deliverable to slice
