---
id: TC-183
title: binary_compiles_x86
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

`cargo build --release --target x86_64-unknown-linux-musl` completes with zero errors and zero warnings.