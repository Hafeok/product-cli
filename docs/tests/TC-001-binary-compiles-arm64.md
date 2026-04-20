---
id: TC-001
title: binary_compiles_arm64
type: smoke
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
runner-args: "tc_001_binary_compiles_arm64"
last-run: 2026-04-14T10:48:19.709127491+00:00
---

`cargo build --release --target aarch64-unknown-linux-gnu` completes with zero errors and zero warnings.