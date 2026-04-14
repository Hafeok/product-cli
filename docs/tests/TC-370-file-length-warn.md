---
id: TC-370
title: file_length_warn
type: scenario
status: unimplemented
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
---

add a 350-line file. Run `file-length.sh`. Assert exit 2. Assert the file name appears in output.