---
id: TC-131
title: metrics_jsonl_merge_conflict_safe
type: scenario
status: unimplemented
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
---

create `metrics.jsonl` with two records on the same line (simulating a bad merge). Assert `product metrics trend` handles it gracefully with a W-class warning.