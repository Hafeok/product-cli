---
id: TC-067
title: atomic_write_interrupted
type: scenario
status: passing
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
runner: cargo-test
runner-args: "tc_067_atomic_write_interrupted"
---

simulate a write failure after temp file creation (inject error before rename). Assert the target file is unchanged. Assert the temp file is deleted.