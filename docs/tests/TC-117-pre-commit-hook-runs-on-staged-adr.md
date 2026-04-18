---
id: TC-117
title: pre_commit_hook_runs_on_staged_adr
type: scenario
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_117_pre_commit_hook_runs_on_staged_adr"
last-run: 2026-04-18T10:41:48.879855342+00:00
last-run-duration: 0.2s
---

stage an ADR file with a missing Rejected alternatives section. Run the pre-commit hook. Assert the structural finding is printed to stdout. Assert exit code 0 (advisory).