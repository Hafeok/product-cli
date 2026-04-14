---
id: TC-002
title: binary_compiles_x86
type: scenario
status: passing
validates:
  features:
  - FT-001
  - FT-012
  - FT-013
  adrs:
  - ADR-001
phase: 1
runner: cargo-test
runner-args: "tc_002_binary_compiles_x86"
last-run: 2026-04-14T10:48:19.709127491+00:00
---

`cargo build --release --target x86_64-unknown-linux-musl` completes with zero errors and zero warnings.