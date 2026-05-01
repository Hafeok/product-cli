---
id: TC-670
title: session ST-015 change-append-deduplicates
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
runner-args: tc_670_session_st_015_change_append_deduplicates
last-run: 2026-04-30T09:23:51.770116705+00:00
last-run-duration: 0.4s
---

Session ST-015 — appending a value that already exists is idempotent. Validates deduplication semantics of the append op.