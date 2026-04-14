---
id: TC-027
title: exit_code_clean
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
runner-args: "tc_027_exit_code_clean"
last-run: 2026-04-14T13:40:28.280537041+00:00
---

run `product graph check` on a fully consistent repository. Assert exit code 0.