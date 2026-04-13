---
id: TC-085
title: validates.features
type: scenario
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_085_validates_features"
---

Features extracted from PRD have empty adrs and tests lists. Migration does not infer feature-to-ADR or feature-to-test links (requires human review per ADR-017).