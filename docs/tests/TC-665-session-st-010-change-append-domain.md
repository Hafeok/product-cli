---
id: TC-665
title: session ST-010 change-append-domain
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
runner-args: tc_665_session_st_010_change_append_domain
last-run: 2026-04-22T11:46:15.496146315+00:00
last-run-duration: 0.2s
---

Session ST-010 — change appends a domain to a feature created in a prior request. Validates the append op on array fields.