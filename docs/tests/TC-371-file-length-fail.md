---
id: TC-371
title: file_length_fail
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_371_file_length_fail"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

add a 450-line file. Run `file-length.sh`. Assert exit 1. Assert the file name and line count appear in output.