---
id: TC-415
title: product agent-init --watch regenerates on graph change
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_415_product_agent_init_watch_regenerates_on_graph_change"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Start `product agent-init --watch` in background. Modify a feature file's front-matter (e.g. change status). Assert `AGENT.md` is regenerated within 2 seconds with updated repository state reflecting the change. Kill the watch process and assert clean exit.