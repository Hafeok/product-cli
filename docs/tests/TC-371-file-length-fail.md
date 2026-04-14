---
id: TC-371
title: file_length_fail
type: scenario
status: unimplemented
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
---

add a 450-line file. Run `file-length.sh`. Assert exit 1. Assert the file name and line count appear in output.