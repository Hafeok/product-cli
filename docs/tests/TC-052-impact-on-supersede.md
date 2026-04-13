---
id: TC-052
title: impact_on_supersede
type: scenario
status: passing
validates:
  features:
  - FT-011
  - FT-016
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_052_impact_on_supersede"
---

run `product adr status ADR-002 superseded --by ADR-013`. Assert impact summary is printed to stdout before the status change is committed.