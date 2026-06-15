---
id: TC-896
title: author domain print-prompt emits facilitation prompt
type: scenario
status: passing
validates:
  features:
  - FT-109
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_896_author_domain_print_prompt
---

## Scenario — print-prompt emits the facilitation prompt

**Given** any repository,
**When** the user runs `product author domain acme --print-prompt`,
**Then** the process exits 0 and stdout contains the What-capture facilitation
prompt naming the product `acme` and the `session_finalize` tool.

## Validates

- FT-109 — product author domain — facilitated What-capture MCP session
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
