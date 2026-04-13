---
id: TC-131
title: metrics_jsonl_merge_conflict_safe
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: "tc_131_metrics_jsonl_merge_conflict_safe"
last-run: 2026-04-13T14:27:30.366814571+00:00
---

create `metrics.jsonl` with two records on the same line (simulating a bad merge). Assert `product metrics trend` handles it gracefully with a W-class warning.