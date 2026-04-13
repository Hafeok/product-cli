---
id: TC-012
title: markdown_passthrough
type: scenario
status: passing
validates:
  features:
  - FT-001
  - FT-002
  - FT-007
  adrs:
  - ADR-004
phase: 1
runner: cargo-test
runner-args: "tc_012_markdown_passthrough"
---

a markdown file with code blocks, tables, and nested lists. Assert the context bundle output preserves these structures verbatim.