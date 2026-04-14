---
id: TC-192
title: markdown_front_matter_strip
type: scenario
status: unimplemented
validates:
  features: 
  - FT-001
  - FT-002
  - FT-007
  adrs:
  - ADR-004
phase: 1
---

read a markdown file with front-matter. Assert the context bundle output contains no `---` delimiters and no YAML fields.