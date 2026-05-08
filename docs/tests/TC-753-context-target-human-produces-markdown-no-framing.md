---
id: TC-753
title: context-target-human-produces-markdown-no-framing
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_753_context_target_human_produces_markdown_no_framing
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.6s
---

## Scenario — `context-target-human-produces-markdown-no-framing`

**Given** the built-in `human` template,
**When** the user runs `product context FT-XXX --target human`,
**Then** stdout is plain Markdown with no XML wrappers, no JSON, no YAML envelope, no `<context_bundle>` framing — just headings, prose, and tables.

`[ordering].sections` for `human` excludes `task`, `deliverables`, and `bundle_metrics` — humans read the feature body in declared order.

## Validates

- FT-063 — Per-Model Context Bundle Templates (human-readable default)
- ADR-049 — Per-Model Context Bundle Templates as Data Files