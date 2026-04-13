---
id: TC-087
title: gap_check_no_gaps
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "g003_not_detected_when_present"
---

run `product gap check ADR-001` against a fixture with full TC coverage. Assert exit code 0 and an empty findings array.