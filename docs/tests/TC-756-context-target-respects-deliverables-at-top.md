---
id: TC-756
title: context-target-respects-deliverables-at-top
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_756_context_target_respects_deliverables_at_top
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.4s
---

## Scenario — `context-target-respects-deliverables-at-top`

**Given** a template with `[ordering].deliverables_at_top = true`,
**When** the user runs `product context FT-XXX --target NAME`,
**Then** a flat deliverables list appears at the top of the rendered bundle (after `task` if `critical_first`, otherwise at the very top), in addition to whatever the feature body itself contains.

When `deliverables_at_top = false` (e.g. `human` template), no top-level duplication occurs and deliverables appear only inside the feature body.

## Validates

- FT-063 — Per-Model Context Bundle Templates (`deliverables_at_top` knob)
- ADR-049 — Per-Model Context Bundle Templates as Data Files