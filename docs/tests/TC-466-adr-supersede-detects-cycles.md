---
id: TC-466
title: adr supersede detects cycles
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_466_adr_supersede_detects_cycles"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.2s
---

Create ADR-A, ADR-B, ADR-C. Set ADR-B supersedes ADR-A, ADR-C supersedes ADR-B. Now run `product adr supersede ADR-A --supersedes ADR-C`. Assert exit code 1 and error E004 (supersession cycle detected). Assert no files were modified.