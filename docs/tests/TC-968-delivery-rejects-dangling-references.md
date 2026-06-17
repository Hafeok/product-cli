---
id: TC-968
title: delivery rejects dangling references
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
- stderr
runner: cargo-test
runner-args: tc_968_delivery_rejects_dangling_references
---

## Scenario — a delivery unit pointing at nothing is refused

**Given** a captured What graph with no slice `ghost-slice` and no deliverable
`ghost-feature`,
**When** the user runs `product deliverable new x --slice ghost-slice` and
`product release new R --feature ghost-feature`,
**Then** each exits 1, naming the unresolved reference (`ghost-slice`,
`ghost-feature`) in stderr, and writes nothing.

## Validates

- FT-126 — product deliverable and release — the delivery layer over slices
- ADR-067 — Delivery units are id-pointers — release to deliverable to slice
