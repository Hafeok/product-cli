---
id: TC-030
title: exit_code_ci_pipeline.sh
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-010
  adrs:
  - ADR-009
phase: 1
---

shell script that runs `product graph check` and asserts the pipeline step fails on exit code 1 but passes on exit code 2 with the correct conditional.