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
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

run `product graph coverage --format json`. Assert valid JSON with `features` array, each containing `domains` map with coverage status.