---
id: TC-203
title: context_measure_idempotent
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_203_context_measure_idempotent"
validates:
  features: 
  - FT-011
  adrs:
  - ADR-006
phase: 1
last-run: 2026-04-14T13:57:28.405167723+00:00
---

run `product context FT-001 --measure` twice. Assert `metrics.jsonl` has two entries (one per invocation). Assert front-matter `bundle` block reflects the most recent measurement only.