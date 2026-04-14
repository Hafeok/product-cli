---
id: TC-327
title: drift_scan_returns_adrs
type: scenario
status: unimplemented
validates:
  features: 
  - FT-028
  adrs:
  - ADR-023
phase: 1
---

call `product drift scan src/consensus/raft.rs` on a fixture where ADR-002 has `source-files: [src/consensus/raft.rs]`. Assert ADR-002 is in the result.