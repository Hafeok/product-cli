---
id: TC-117
title: pre_commit_hook_runs_on_staged_adr
type: scenario
status: unimplemented
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
---

stage an ADR file with a missing Rejected alternatives section. Run the pre-commit hook. Assert the structural finding is printed to stdout. Assert exit code 0 (advisory).