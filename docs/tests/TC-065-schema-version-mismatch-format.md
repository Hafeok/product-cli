---
id: TC-065
title: schema_version_mismatch_format
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
runner: cargo-test
runner-args: "tc_065_schema_version_mismatch_format"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

assert error E008 includes the file path, the declared version, the supported version, and the upgrade hint.