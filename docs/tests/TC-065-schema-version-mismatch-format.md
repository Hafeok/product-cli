---
id: TC-065
title: schema_version_mismatch_format
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-020
  adrs:
  - ADR-014
phase: 1
runner: cargo-test
runner-args: "tc_065_schema_version_mismatch_format"
---

assert error E008 includes the file path, the declared version, the supported version, and the upgrade hint.