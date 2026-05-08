---
id: TC-763
title: templates-reset-cannot-touch-builtin
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_763_templates_reset_cannot_touch_builtin
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.3s
---

## Scenario — `templates-reset-cannot-touch-builtin`

**Given** no user override exists for `claude-opus` (it resolves only to the built-in path),
**When** the user runs `product context templates --reset claude-opus`,
**Then** **E029 cannot-reset-builtin** is emitted; no file is deleted; the built-in remains intact.

This invariant — built-in templates are read-only — is asserted by direct file inspection after the reset attempt.

## Validates

- FT-063 — Per-Model Context Bundle Templates (read-only built-ins)
- ADR-049 — Per-Model Context Bundle Templates as Data Files