---
id: TC-556
title: verify_skips_locked_phase_features
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_556_verify_skips_locked_phase_features
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-114 — verify-skips-locked-phase-features

**Validates:** FT-044, ADR-040 (Locked-phase features are skipped with a reason)

### Given

A temp repository with features across two phases: phase 1 features are complete; phase 2 features exist but phase 2 is locked (per the phase gate rules in ADR-034).

### When

`product verify` is run with no arguments.

### Then

- Stage 5 reports each phase-2 feature as `skipped` with `reason: "phase-2-locked"` (or equivalent).
- No phase-2 TC is actually executed.
- Exit code is not determined by the skipped features — only by the phase-1 features' results.
- Pretty output lists skipped TCs under their feature with a bracketed `[phase N locked]` marker.