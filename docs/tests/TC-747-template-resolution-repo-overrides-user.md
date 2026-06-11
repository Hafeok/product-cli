---
id: TC-747
title: template-resolution-repo-overrides-user
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_747_template_resolution_repo_overrides_user
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `template-resolution-repo-overrides-user`

**Given** the same template name `claude-opus` exists in `.product/templates/` (repo) **and** `~/.product/templates/` (user),
**When** the resolver builds the target map,
**Then** the repo-local file is selected and `product context templates --where claude-opus` reports the repo path.

Resolution order is repo → user → built-in; first match wins.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files