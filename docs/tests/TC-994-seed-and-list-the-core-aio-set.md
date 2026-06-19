---
id: TC-994
title: seed and list the core AIO set
type: scenario
status: unimplemented
validates:
  features:
  - FT-134
  adrs:
  - ADR-078
phase: 7
observes:
- graph
- stdout
runner: cargo-test
runner-args: tc_994_seed_and_list_the_core_aio_set
---

## Scenario — the closed-core AIO vocabulary is present and a context of use is declarable

**Given** a captured What graph for a product,
**When** the user runs `product domain list aio`,
**Then** the process exits 0 and stdout lists the closed-core AIOs of the §3.2.2
table — at least `trigger-action`, `single-select`, `multi-select`,
`text-entry`, `numeric-entry`, `date-entry`, `display-value`,
`display-collection`, `navigate`, and `edit` — each present as an `Aio` node in
the graph.

**And when** the user declares a context of use (form factor `phone`, modality
`touch`), **then** a `ContextOfUse` node is added to the What graph and is
surfaced by `product domain list context-of-use`.

## Validates

- FT-134 — Abstract Interaction Object vocabulary and the typed UiStep
- ADR-078 — UI steps are typed against Abstract Interaction Objects; AIOs and CIOs are graph nodes
