---
id: TC-052
title: impact_on_supersede
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
runner-args: "tc_052_impact_on_supersede"
last-run: 2026-04-14T15:03:33.506444091+00:00
---

run `product adr status ADR-002 superseded --by ADR-013`. Assert impact summary is printed to stdout before the status change is committed.