---
id: TC-075
title: parse_evidence_delta_out_of_range
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_075_parse_evidence_delta_out_of_range"
---

Parse an evidence block with delta=1.5 (out of range [0.0, 1.0]). Assert E001 with the file path, line number, and the out-of-range value.