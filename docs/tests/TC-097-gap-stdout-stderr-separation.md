---
id: TC-097
title: gap_stdout_stderr_separation
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_097_gap_stdout_stderr_separation"
---

gap findings are always on stdout. Analysis errors are always on stderr. Verified by piping stdout only and asserting it is valid JSON.