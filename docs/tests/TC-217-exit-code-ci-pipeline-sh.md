---
id: TC-217
title: exit_code_ci_pipeline.sh
type: exit-criteria
status: unimplemented
validates:
  features: 
  - FT-010
  - FT-014
  - FT-026
  adrs:
  - ADR-009
phase: 1
---

shell script that runs `product graph check` and asserts the pipeline step fails on exit code 1 but passes on exit code 2 with the correct conditional.