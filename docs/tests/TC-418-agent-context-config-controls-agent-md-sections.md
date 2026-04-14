---
id: TC-418
title: agent-context config controls AGENT.md sections
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_418_agent_context_config_controls_agent_md_sections"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Set `include-schemas = false` in `[agent-context]` in `product.toml`. Run `product agent-init`. Assert `AGENT.md` does not contain a "Front-Matter Schemas" section. Re-enable `include-schemas = true`, re-run, assert the section reappears. Repeat for `include-repo-state`, `include-domains`, and `include-tool-guide`.