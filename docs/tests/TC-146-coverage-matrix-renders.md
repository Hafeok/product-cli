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
---

run `product graph coverage` on a fixture with known coverage state. Assert output contains all features and all domains. Assert correct ✓/~/·/✗ symbols.