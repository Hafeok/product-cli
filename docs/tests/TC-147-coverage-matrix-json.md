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
last-run: 2026-04-30T09:23:24.144322835+00:00
last-run-duration: 0.3s
---

run `product graph coverage --format json`. Assert valid JSON with `features` array, each containing `domains` map with coverage status.