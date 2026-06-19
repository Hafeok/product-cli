---
id: TC-1019
title: product guide orients a new user
type: scenario
status: passing
validates:
  features:
  - FT-143
  adrs:
  - ADR-088
phase: 8
observes:
- stdout
- exit-code
runner: cargo-test
runner-args: tc_1019_product_guide_orients_a_new_user
last-run: 2026-06-19T15:22:00.175149544+00:00
last-run-duration: 7.5s
---

## Scenario — guide names the stage and the next step on a fresh repo

**Given** a freshly initialised repo with no framework graph,
**When** the user runs `product guide`,
**Then** the process exits 0 and stdout shows the journey checklist with
`[ ] Captured a What model` and suggests the next step (`product author
domain …`).

**And when** the user runs `product --format json guide`, **then** the process
exits 0 and the JSON carries the structured stage (`capture-what`).

## Validates

- FT-143 — product guide — state-aware framework-graph onboarding
- ADR-088 — framework-graph onboarding is a derived guide plus a signposted, seedable init