---
id: TC-123
title: drift_scan_returns_adrs
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-023
phase: 1
runner: cargo-test
runner-args: "tc_123_drift_scan_returns_adrs"
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

call `product drift scan src/consensus/raft.rs` on a fixture where ADR-002 has `source-files: [src/consensus/raft.rs]`. Assert ADR-002 is in the result.