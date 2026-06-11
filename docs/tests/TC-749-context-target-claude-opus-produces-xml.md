---
id: TC-749
title: context-target-claude-opus-produces-xml
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_749_context_target_claude_opus_produces_xml
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `context-target-claude-opus-produces-xml`

**Given** the built-in `claude-opus` template (`[format].structure = "xml"`),
**When** the user runs `product context FT-XXX --target claude-opus`,
**Then** stdout contains a `<context_bundle>` root element with child elements named per `[ordering].sections` (e.g. `<task>`, `<feature>`, `<deliverables>`, `<governing_adrs>`, ...).

`[format.xml].include_attributes = true` means metadata appears as XML attributes.

## Validates

- FT-063 — Per-Model Context Bundle Templates (XML rendering)
- ADR-049 — Per-Model Context Bundle Templates as Data Files