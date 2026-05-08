---
id: TC-759
title: templates-list-shows-all-resolved-templates
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_759_templates_list_shows_all_resolved_templates
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.4s
---

## Scenario — `templates-list-shows-all-resolved-templates`

**Given** the six built-in templates plus one user template at `~/.product/templates/team-bundle.toml` and one repo template at `.product/templates/pr-review.toml`,
**When** the user runs `product context templates`,
**Then** stdout lists all eight names with their descriptions and source markers (`(built-in)`, `(user)`, `(repo)`); a `Default target:` footer reports the currently configured default and where it came from (`from product.toml` or `fallback`).

## Validates

- FT-063 — Per-Model Context Bundle Templates (`templates` list command)
- ADR-049 — Per-Model Context Bundle Templates as Data Files