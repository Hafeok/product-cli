---
id: TC-748
title: template-resolution-user-overrides-builtin
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_748_template_resolution_user_overrides_builtin
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `template-resolution-user-overrides-builtin`

**Given** a template name `claude-opus` exists in `~/.product/templates/` and in `$PRODUCT_INSTALL/templates/`, with no repo-local copy,
**When** the resolver runs,
**Then** the user-level file is selected; `product context templates --where claude-opus` reports the user path.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files