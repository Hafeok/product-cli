---
id: TC-922
title: domain context unknown node is a clear error
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
- stderr
runner: cargo-test
runner-args: tc_922_domain_context_unknown_node_is_a_clear_error
---

## Scenario — an unknown focus id is a clear error

**Given** a captured What graph,
**When** the user runs `product domain context ghost` for an id not in the
graph,
**Then** the process exits 1 and stderr reports 'no node with id'.

## Validates

- FT-112 — product domain context — assemble an LLM context bundle from the What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
