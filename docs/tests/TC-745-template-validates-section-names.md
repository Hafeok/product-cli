---
id: TC-745
title: template-validates-section-names
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_745_template_validates_section_names
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `template-validates-section-names`

**Given** a template with `[ordering].sections = ["task", "meta", "feature"]` where `meta` is not in the recognised section list,
**When** validation runs,
**Then** **E030 invalid-template** names `meta` as the offending entry and lists the closed allowlist, and the template is excluded.

Closed allowlist: `task`, `feature`, `deliverables`, `governing_adrs`, `test_criteria`, `dependencies`, `linked_documentation`, `constraints`, `bundle_metrics`.

## Validates

- FT-063 — Per-Model Context Bundle Templates
- ADR-049 — Per-Model Context Bundle Templates as Data Files