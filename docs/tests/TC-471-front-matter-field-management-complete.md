---
id: TC-471
title: front-matter field management complete
type: exit-criteria
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_471_front_matter_field_management_complete"
last-run: 2026-04-15T11:45:30.776758337+00:00
last-run-duration: 0.3s
---

Exit criteria for FT-038:

1. All front-matter fields on features, ADRs, and TCs are editable through both CLI commands and MCP tools
2. No field requires manual YAML editing to set or modify
3. All validation rules (E012, E011, E004, E001) are enforced on every mutation
4. The author-feature and author-adr system prompts reference the new tools
5. A complete authoring session (scaffold → link → domain → acknowledge → scope → supersede → runner) can be performed entirely through MCP tools without touching files directly