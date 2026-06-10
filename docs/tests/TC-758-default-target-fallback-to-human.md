---
id: TC-758
title: default-target-fallback-to-human
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_758_default_target_fallback_to_human
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `default-target-fallback-to-human`

**Given** `product.toml` without a `[context]` section (no `default-target` set),
**When** the user runs `product context FT-XXX`,
**Then** the bundle is rendered using the `human` template (Markdown, no framing).

Confirms the backward-compat invariant: `product context FT-XXX` without flags on a fresh repo produces terminal-readable Markdown.

## Validates

- FT-063 — Per-Model Context Bundle Templates (fallback default)
- ADR-049 — Per-Model Context Bundle Templates as Data Files