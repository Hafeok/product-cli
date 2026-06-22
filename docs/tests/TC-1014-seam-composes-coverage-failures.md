---
id: TC-1014
title: seam composes coverage failures
type: scenario
status: passing
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
runner-args: tc_1014_seam_composes_coverage_failures
last-run: 2026-06-22T13:02:02.406096890+00:00
last-run-duration: 0.7s
---

## Scenario — every coverage gap is reported independently, never collapsed into one fail

**Given** a captured What graph and How in which a `UiStep` has, simultaneously,
a missing reification for one (AIO, context) pair, an uncovered surfaced-
projection state, and an unresolved (content key, locale) pair,
**When** the user runs `product seam` on that step,
**Then** the process exits non-zero and the composite verdict **lists every
failing sub-check** — the reification-coverage gap, the state-coverage gap, and
the content-coverage gap each named separately with its basis — rather than
emitting a single opaque failure. The seam composes the per-phase checks
(FT-136/FT-138/FT-139) without hiding which obligation was unmet.

## Validates

- FT-140 — The seam verification — screen against UI step
- ADR-084 — The seam verification confirms a screen and its UI step agree