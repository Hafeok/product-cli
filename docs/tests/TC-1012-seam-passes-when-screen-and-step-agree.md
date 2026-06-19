---
id: TC-1012
title: seam passes when screen and step agree
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
runner-args: tc_1012_seam_passes_when_screen_and_step_agree
---

## Scenario — a fully-agreeing screen passes the seam with a level-and-basis verdict

**Given** a captured What graph with a flow whose `UiStep` `ReviewOrder`
surfaces the `OrderSummary` projection, offers the `ConfirmOrder` command, types
every interaction against an `Aio`, covers every projection state, references
content keys that resolve in the content store for each declared locale, and
whose accessibility obligations are discharged,
**And** a How in which every referenced AIO reifies to a CIO for each declared
context,
**When** the user runs `product seam ReviewOrder`,
**Then** the process exits 0 and the seam verdict **passes**, reporting the
conformance **level and its basis** (which sub-checks were satisfied and how) —
not a bare pass.

## Validates

- FT-140 — The seam verification — screen against UI step
- ADR-084 — The seam verification confirms a screen and its UI step agree
