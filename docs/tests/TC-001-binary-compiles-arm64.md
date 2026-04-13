---
id: TC-001
title: binary_compiles_arm64
type: scenario
status: unimplemented
validates:
  features:
  - FT-001
  - FT-012
  - FT-013
  adrs:
  - ADR-001
phase: 1
---

`cargo build --release --target aarch64-unknown-linux-gnu` completes with zero errors and zero warnings.