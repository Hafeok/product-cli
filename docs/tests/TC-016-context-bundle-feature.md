---
id: TC-016
title: context_bundle_feature
type: scenario
status: passing
validates:
  features:
  - FT-011
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_016_context_bundle_feature"
last-run: 2026-04-14T13:57:28.405167723+00:00
---

call `product context FT-001` on a repository with FT-001 linked to ADR-001, ADR-002, and TC-001. Assert the output contains the feature content, both ADR contents, and the test criterion content, in the correct order.