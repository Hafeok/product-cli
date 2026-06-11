---
id: TC-762
title: templates-reset-removes-user-override
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_762_templates_reset_removes_user_override
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `templates-reset-removes-user-override`

**Given** a user override at `~/.product/templates/claude-opus.toml`,
**When** the user runs `product context templates --reset claude-opus`,
**Then** the user file is deleted; `product context templates --where claude-opus` now reports the built-in path.

The reset is a single `fs::remove_file` under advisory lock. Repo-local overrides are never auto-deleted.

## Validates

- FT-063 — Per-Model Context Bundle Templates (`--reset`)
- ADR-049 — Per-Model Context Bundle Templates as Data Files