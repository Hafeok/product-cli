---
id: TC-059
title: error_stdout_clean
type: scenario
status: unimplemented
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
---

run any command that produces warnings but no errors. Assert stdout contains only the command's normal output. Assert warnings are on stderr only.