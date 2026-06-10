---
id: TC-750
title: context-target-gpt-4-markdown-produces-markdown
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_750_context_target_gpt_4_markdown_produces_markdown
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `context-target-gpt-4-markdown-produces-markdown`

**Given** the built-in `gpt-4-markdown` template (`[format].structure = "markdown"`),
**When** the user runs `product context FT-XXX --target gpt-4-markdown`,
**Then** stdout is well-formed Markdown using `[format.markdown].heading_levels = "h2-h3"` (top-level sections at `##`, sub-sections at `###`) and `table_format = "github"`.

No XML or JSON envelope is emitted.

## Validates

- FT-063 — Per-Model Context Bundle Templates (Markdown rendering)
- ADR-049 — Per-Model Context Bundle Templates as Data Files