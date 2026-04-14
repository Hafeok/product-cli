---
id: TC-030
title: exit_code_ci_pipeline.sh
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
runner-args: "tc_030_exit_code_ci_pipeline"
last-run: 2026-04-14T15:02:41.236412349+00:00
---

shell script that runs `product graph check` and asserts the pipeline step fails on exit code 1 but passes on exit code 2 with the correct conditional.