---
id: TC-761
title: templates-show-prints-template-toml
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_761_templates_show_prints_template_toml
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.3s
---

## Scenario — `templates-show-prints-template-toml`

**Given** any resolved template `NAME`,
**When** the user runs `product context templates --show NAME`,
**Then** stdout is the full TOML content of the resolved template file, byte-identical to the file on disk.

This is the supported workflow for forking a built-in: `product context templates --show claude-opus > ~/.product/templates/my.toml`.

## Validates

- FT-063 — Per-Model Context Bundle Templates (`--show`)
- ADR-049 — Per-Model Context Bundle Templates as Data Files