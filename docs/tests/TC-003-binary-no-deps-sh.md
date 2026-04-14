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
last-run: 2026-04-14T10:48:19.709127491+00:00
---

`ldd product` on the Linux binary reports no dynamic dependencies beyond `libc`. Any other line is a test failure.