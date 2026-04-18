---
id: TC-146
title: coverage_matrix_renders
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
runner-args: "tc_146_coverage_matrix_renders"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

run `product graph coverage` on a fixture with known coverage state. Assert output contains all features and all domains. Assert correct ✓/~/·/✗ symbols.