---
id: TC-369
title: file_length_passes
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_369_file_length_passes"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

create a temp Rust project where all files are under 300 lines. Run `file-length.sh`. Assert exit 0.