---
id: TC-004
title: cargo build --release
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
runner-args: "tc_004_cargo_build_release"
---