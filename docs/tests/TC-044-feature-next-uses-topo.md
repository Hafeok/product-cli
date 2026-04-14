---
id: TC-044
title: feature_next_uses_topo
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
runner-args: "tc_044_feature_next_uses_topo"
last-run: 2026-04-14T14:53:21.175394484+00:00
---

FT-001 complete, FT-002 depends-on FT-001 (in-progress), FT-003 no dependencies (planned). Assert `product feature next` returns FT-002, not FT-003.