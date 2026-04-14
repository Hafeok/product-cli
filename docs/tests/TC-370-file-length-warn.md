---
id: TC-370
title: file_length_warn
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_370_file_length_warn"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

add a 350-line file. Run `file-length.sh`. Assert exit 2. Assert the file name appears in output.