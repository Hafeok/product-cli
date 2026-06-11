---
id: TC-754
title: context-target-omits-sections-not-in-ordering-list
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_754_context_target_omits_sections_not_in_ordering_list
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `context-target-omits-sections-not-in-ordering-list`

**Given** a template whose `[ordering].sections` lists only `["task", "feature", "test_criteria"]` (a minimal target),
**When** the user runs `product context FT-XXX --target minimal`,
**Then** the rendered bundle contains only those three sections; `governing_adrs`, `dependencies`, `linked_documentation`, `constraints`, and `bundle_metrics` are absent from the output.

Section omission is the v1 mechanism for "minimal" bundles aimed at small-context models or scratch use cases.

## Validates

- FT-063 — Per-Model Context Bundle Templates (section omission)
- ADR-049 — Per-Model Context Bundle Templates as Data Files