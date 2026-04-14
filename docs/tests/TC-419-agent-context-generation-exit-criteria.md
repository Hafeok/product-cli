---
id: TC-419
title: Agent context generation exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_419_agent_context_generation_exit_criteria"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product schema --all` — output contains feature, ADR, test criterion, and dependency schemas with all fields documented. Run `product agent-init` — `AGENT.md` is created with all five sections (protocol, repo state, schemas, domains, tool guide). Modify a feature status, re-run `product agent-init` — repo state section reflects the change. Call `product_schema feature` and `product_agent_context` via MCP — responses match CLI output. Set `include-schemas = false` in `[agent-context]`, re-run — schemas section absent.