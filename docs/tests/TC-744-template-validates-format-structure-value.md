---
id: TC-744
title: template-validates-format-structure-value
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_744_template_validates_format_structure_value
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.5s
---

## Scenario — `template-validates-format-structure-value`

**Given** a template with `[format].structure = "toml"` (an unrecognised value),
**When** validation runs,
**Then** **E030 invalid-template** lists the unrecognised value and the allowed set `{xml, markdown, yaml, json, plain}`, and the template is excluded from the targets list.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files