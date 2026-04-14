---
id: TC-028
title: exit_code_broken_link
type: exit-criteria
status: passing
validates:
  features:
  - FT-010
  - FT-014
  adrs:
  - ADR-009
phase: 1
runner: cargo-test
runner-args: "tc_028_exit_code_broken_link"
last-run: 2026-04-14T13:40:28.280537041+00:00
---

add a feature that references a non-existent ADR. Assert exit code 1.