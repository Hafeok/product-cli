---
id: TC-203
title: context_measure_idempotent
type: scenario
status: unimplemented
validates:
  features: 
  - FT-011
  adrs:
  - ADR-006
phase: 1
---

run `product context FT-001 --measure` twice. Assert `metrics.jsonl` has two entries (one per invocation). Assert front-matter `bundle` block reflects the most recent measurement only.