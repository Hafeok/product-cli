---
id: TC-668
title: session ST-013 change-body-mutation
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
runner-args: tc_668_session_st_013_change_body_mutation
last-run: 2026-04-30T09:23:51.770116705+00:00
last-run-duration: 0.5s
---

Session ST-013 — change rewrites the prose body while preserving front-matter. Validates the virtual body field.