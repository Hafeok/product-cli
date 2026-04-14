---
id: TC-415
title: product agent-init --watch regenerates on graph change
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

Start `product agent-init --watch` in background. Modify a feature file's front-matter (e.g. change status). Assert `AGENT.md` is regenerated within 2 seconds with updated repository state reflecting the change. Kill the watch process and assert clean exit.
