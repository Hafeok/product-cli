---
id: TC-746
title: invalid-template-excluded-from-targets-list
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_746_invalid_template_excluded_from_targets_list
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.5s
---

## Scenario — `invalid-template-excluded-from-targets-list`

**Given** a workspace with one valid template (`good.toml`) and one invalid template (`bad.toml`, missing `[ordering]`),
**When** the user runs `product context templates`,
**Then** `good` appears in the output and `bad` does not; a stderr warning lists `bad` as excluded with the validation reason.

The binary continues to serve targets other than `bad` without erroring.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files