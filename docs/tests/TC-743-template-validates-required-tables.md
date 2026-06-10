---
id: TC-743
title: template-validates-required-tables
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_743_template_validates_required_tables
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `template-validates-required-tables`

**Given** a template TOML missing one of `[template]`, `[format]`, or `[ordering]`,
**When** template validation runs at startup,
**Then** **E030 invalid-template** is emitted with a finding pointing to the missing table, and the template is excluded from the targets list.

The binary still runs on other targets — invalid templates are warnings, not hard errors.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files