---
id: TC-1009
title: AIO reifies to different CIOs by context
type: scenario
status: passing
validates:
  features:
  - FT-139
  adrs:
  - ADR-083
phase: 7
observes:
- graph
- stdout
runner: cargo-test
runner-args: tc_1009_aio_reifies_to_different_cios_by_context
last-run: 2026-06-21T19:06:43.421096467+00:00
last-run-duration: 18.0s
---

## Scenario — one AIO, many CIOs by context, the What unchanged

**Given** a design system whose closed CIO catalog contains `segmented-control`
and `searchable-list`,
**And** two reification rules for the `single-select` AIO — one `in_context`
{form_factor: tablet, options: few} that `reifies` `segmented-control` with the
rationale "few options, ample width — direct choice beats a menu", and one
`in_context` {form_factor: phone, options: many} that `reifies` `searchable-list`
with the rationale "a phone has no room for many side-by-side options",
**When** the user shows the reification for `single-select`,
**Then** the process exits 0 and stdout reports both rules with their contexts,
CIOs, and rationale — the same `single-select` in the What reifies to two
different controls, and the AIO node itself is unchanged across both.

## Validates

- FT-139 — Design system and reification rules
- ADR-083 — Screens bind to a design system; AIOs reify to CIOs by context of use