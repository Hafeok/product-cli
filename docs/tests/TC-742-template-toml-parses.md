---
id: TC-742
title: template-toml-parses
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_742_template_toml_parses
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.5s
---

## Scenario — `template-toml-parses`

**Given** a well-formed template TOML at `.product/templates/sample.toml` with required `[template]`, `[format]`, `[ordering]` tables,
**When** the template loader resolves available targets,
**Then** `sample` appears in `product context templates` output and `--target sample` renders without error.

Asserts the happy-path TOML parse over the `serde::Deserialize` template type. Pure parser test.

## Validates

- FT-063 — Per-Model Context Bundle Templates (template loader)
- ADR-049 — Per-Model Context Bundle Templates as Data Files