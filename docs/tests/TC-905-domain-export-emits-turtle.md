---
id: TC-905
title: domain export emits turtle
type: scenario
status: passing
validates:
  features:
  - FT-110
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_905_domain_export_emits_turtle
---

## Scenario — export the captured graph as Turtle

**Given** a built What graph,
**When** the user runs `product domain export`,
**Then** the process exits 0 and stdout is Turtle containing the `@prefix pf:`
header, `d:Order a pf:Entity`, and `pf:changes d:Order`.

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
