---
id: TC-289
title: gap_suppress_mutates_baseline
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

run `product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred"`. Assert `gaps.json` contains the suppression with the reason, timestamp, and current commit hash.