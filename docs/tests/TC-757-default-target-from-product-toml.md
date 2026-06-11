---
id: TC-757
title: default-target-from-product-toml
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_757_default_target_from_product_toml
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `default-target-from-product-toml`

**Given** `product.toml` containing `[context]\ndefault-target = "claude-opus"`,
**When** the user runs `product context FT-XXX` (no `--target` flag),
**Then** the bundle is rendered using the `claude-opus` template (XML output).

Confirms the config falls through to the template layer correctly.

## Validates

- FT-063 — Per-Model Context Bundle Templates (default target config)
- ADR-049 — Per-Model Context Bundle Templates as Data Files