---
id: TC-418
title: agent-context config controls AGENT.md sections
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Set `include-schemas = false` in `[agent-context]` in `product.toml`. Run `product agent-init`. Assert `AGENT.md` does not contain a "Front-Matter Schemas" section. Re-enable `include-schemas = true`, re-run, assert the section reappears. Repeat for `include-repo-state`, `include-domains`, and `include-tool-guide`.
