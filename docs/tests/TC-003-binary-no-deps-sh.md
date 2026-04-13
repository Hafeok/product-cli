---
id: TC-003
title: binary_no_deps.sh
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
runner-args: "tc_003_binary_no_deps"
---

`ldd product` on the Linux binary reports no dynamic dependencies beyond `libc`. Any other line is a test failure.