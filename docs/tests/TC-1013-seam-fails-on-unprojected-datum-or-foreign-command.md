---
id: TC-1013
title: seam fails on unprojected datum or foreign command
type: scenario
status: unimplemented
validates:
  features:
  - FT-140
  adrs:
  - ADR-084
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1013_seam_fails_on_unprojected_datum_or_foreign_command
---

## Scenario — a screen that needs unsupplied data or issues an unaccepted command fails the seam

**Given** a captured What graph with a `UiStep` `ReviewOrder` whose flow's read
model does **not** project a field the page displays,
**When** the user runs `product seam ReviewOrder`,
**Then** the process exits non-zero and the seam verdict fails on the
**datum-projected** sub-check, naming the offending datum and the read model
that should supply it.

**And given** instead a `UiStep` that offers a control issuing a command the step
cannot accept (a command no Decider `handles` at this step), **when** the seam
runs, **then** it exits non-zero and fails on the **control-maps-to-command**
sub-check, naming the offending control and command.

## Validates

- FT-140 — The seam verification — screen against UI step
- ADR-084 — The seam verification confirms a screen and its UI step agree
