---
id: TC-045
title: context_depth_2
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_045_context_depth_2"
last-run: 2026-04-14T14:53:21.175394484+00:00
---

FT-001 linked to ADR-002; ADR-002 also linked to FT-004; FT-004 linked to TC-009. Assert `product context FT-001 --depth 2` includes TC-009 and FT-004. Assert `product context FT-001 --depth 1` does not.