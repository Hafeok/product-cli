---
id: TC-1027
title: render contract projects page graph and aui
type: scenario
status: passing
validates:
  features:
  - FT-146
  adrs:
  - ADR-085
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1027_render_contract_projects_page_graph_and_aui
last-run: 2026-06-22T19:16:33.567315222+00:00
last-run-duration: 0.5s
---

## Scenario — a flow projects to a walkable render contract

**Given** a captured What graph with an application root whose destinations
include a flow's entry page, a flow listing its pages, and UI steps that surface
a projection (typed against an AIO) and offer a command that transitions to the
next screen,
**When** the user runs `product preview render-contract <flow>`,
**Then** the process exits 0 and emits a `contract_version: "preview-0"` JSON
document carrying the application **root** destinations, the **flow** entry and
pages, and one **screen** per UI step — each with its projection, declared state
space, and `elements` typed by AIO and carrying the WCAG obligations inherited
from that AIO. The contract is walkable from itself alone.

## Validates

- FT-146 — render contract projection
- ADR-085 — preview profiles at the What/How boundary