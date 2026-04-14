---
id: TC-411
title: AGENT.md contains current repository state summary
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

Run `product agent-init`. Assert the "Current Repository State" section shows feature count, ADR count, and TC status counts that match the output of `product status`. Assert phase gate status is included.
