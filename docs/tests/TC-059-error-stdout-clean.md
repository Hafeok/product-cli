---
id: TC-059
title: error_stdout_clean
type: scenario
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_059_error_stdout_clean"
---

run any command that produces warnings but no errors. Assert stdout contains only the command's normal output. Assert warnings are on stderr only.