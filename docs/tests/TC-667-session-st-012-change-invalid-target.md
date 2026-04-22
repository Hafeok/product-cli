---
id: TC-667
title: session ST-012 change-invalid-target
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
runner-args: tc_667_session_st_012_change_invalid_target
last-run: 2026-04-22T11:30:45.579976087+00:00
last-run-duration: 0.2s
---

Session ST-012 — change against a non-existent target ID fails with E002, docs/ is byte-identical after. Validates atomicity of validation.