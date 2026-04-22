---
id: TC-673
title: session ST-052 verify-failing-tc-stays-in-progress
type: session
status: passing
validates:
  features:
  - FT-043
  - FT-044
  adrs:
  - ADR-018
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_673_session_st_052_verify_failing_tc_stays_in_progress
last-run: 2026-04-22T11:46:15.496146315+00:00
last-run-duration: 0.2s
---

Session ST-052 — a failing TC runner keeps the feature off complete, records TC failing, emits no completion tag.