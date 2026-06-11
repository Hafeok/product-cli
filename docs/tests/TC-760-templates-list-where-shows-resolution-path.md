---
id: TC-760
title: templates-list-where-shows-resolution-path
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_760_templates_list_where_shows_resolution_path
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `templates-list-where-shows-resolution-path`

**Given** the same workspace as TC-759,
**When** the user runs `product context templates --where`,
**Then** stdout lists each resolved template name and its absolute path on disk, e.g.:

```
claude-opus       /usr/local/share/product/templates/claude-opus.toml
team-bundle       /home/emil/.product/templates/team-bundle.toml
pr-review         /home/emil/repos/picloud/.product/templates/pr-review.toml
```

## Validates

- FT-063 — Per-Model Context Bundle Templates (`--where`)
- ADR-049 — Per-Model Context Bundle Templates as Data Files