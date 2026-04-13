---
id: TC-011
title: markdown_front_matter_strip
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
runner-args: "tc_011_markdown_front_matter_strip"
---

read a markdown file with front-matter. Assert the context bundle output contains no `---` delimiters and no YAML fields.