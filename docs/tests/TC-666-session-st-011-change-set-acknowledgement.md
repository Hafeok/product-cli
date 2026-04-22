---
id: TC-666
title: session ST-011 change-set-acknowledgement
type: session
status: passing
validates:
  features:
  - FT-041
  - FT-043
  adrs:
  - ADR-018
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_666_session_st_011_change_set_acknowledgement
last-run: 2026-04-22T12:59:08.455929045+00:00
last-run-duration: 0.3s
---

Session ST-011 — change sets a nested domains-acknowledged.<ADR> entry. Validates dot-notation mutation of map fields.