---
id: TC-411
title: AGENT.md contains current repository state summary
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_411_agent_md_contains_current_repository_state_summary"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init`. Assert the "Current Repository State" section shows feature count, ADR count, and TC status counts that match the output of `product status`. Assert phase gate status is included.