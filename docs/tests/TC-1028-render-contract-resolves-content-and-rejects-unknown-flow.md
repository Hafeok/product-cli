---
id: TC-1028
title: render contract resolves content and rejects unknown flow
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
runner-args: tc_1028_render_contract_resolves_content_and_rejects_unknown_flow
last-run: 2026-06-22T19:16:33.567315222+00:00
last-run-duration: 0.4s
---

## Scenario — content resolves per locale; an unknown flow is named

**Given** a captured What graph whose UI step references a content key, and a
content store resolving that key in a claimed locale,
**When** the user runs `product preview render-contract <flow> --locale <loc>`,
**Then** the process exits 0 and the contract's `content_store` block resolves
the referenced key to its string for that locale — never a literal.

**And given** a flow id that does not exist in the graph, **when** the user runs
the command, **then** the process exits non-zero and names the missing flow.

## Validates

- FT-146 — render contract projection
- ADR-085 — preview profiles at the What/How boundary