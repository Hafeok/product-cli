---
id: TC-921
title: domain context depth controls reach
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
runner-args: tc_921_domain_context_depth_controls_reach
---

## Scenario — depth bounds the neighbourhood

**Given** an Order entity related (one hop) to an Invoice entity that lives in
a different context,
**When** the user runs `product domain context Order` at `--depth 1` versus
`--depth 2`,
**Then** both exit 0; the Invoice (two hops away, across the relation) is
absent from the depth-1 bundle stdout but present in the depth-2 bundle stdout.

## Validates

- FT-112 — product domain context — assemble an LLM context bundle from the What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
