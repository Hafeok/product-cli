---
id: TC-147
title: coverage_matrix_json
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_147_coverage_matrix_json"
last-run: 2026-04-13T14:12:26.396687298+00:00
---

run `product graph coverage --format json`. Assert valid JSON with `features` array, each containing `domains` map with coverage status.