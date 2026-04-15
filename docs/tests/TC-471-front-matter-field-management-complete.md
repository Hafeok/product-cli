---
id: TC-471
title: front-matter field management complete
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Exit criteria for FT-038:

1. All front-matter fields on features, ADRs, and TCs are editable through both CLI commands and MCP tools
2. No field requires manual YAML editing to set or modify
3. All validation rules (E012, E011, E004, E001) are enforced on every mutation
4. The author-feature and author-adr system prompts reference the new tools
5. A complete authoring session (scaffold → link → domain → acknowledge → scope → supersede → runner) can be performed entirely through MCP tools without touching files directly