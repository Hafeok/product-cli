---
id: TC-755
title: context-target-orders-sections-as-template-specifies
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_755_context_target_orders_sections_as_template_specifies
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `context-target-orders-sections-as-template-specifies`

**Given** two templates that include the same set of sections but in different orders (e.g. `claude-opus` vs. `human`),
**When** the user runs `product context FT-XXX --target claude-opus` and `product context FT-XXX --target human`,
**Then** each output emits sections in the exact order declared by its template's `[ordering].sections` list.

For `claude-opus`: `task` first (critical-first). For `human`: `feature` first (no critical-first reordering).

## Validates

- FT-063 — Per-Model Context Bundle Templates (section ordering)
- ADR-049 — Per-Model Context Bundle Templates as Data Files