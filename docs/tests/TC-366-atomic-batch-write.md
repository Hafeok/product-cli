---
id: TC-366
title: atomic_batch_write
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_366_atomic_batch_write"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

inject a write failure midway through a multi-file inference batch. Assert all-or-nothing: either all files updated or none. Assert no partial state.