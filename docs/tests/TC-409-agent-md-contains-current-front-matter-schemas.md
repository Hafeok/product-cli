---
id: TC-409
title: AGENT.md contains current front-matter schemas
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_409_agent_md_contains_current_front_matter_schemas"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init`. Assert the generated `AGENT.md` contains a "Front-Matter Schemas" section with subsections for feature, ADR, test criterion, and dependency schemas. Assert schema content matches `product schema --all` output.