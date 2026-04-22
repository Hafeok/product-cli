---
id: TC-679
title: session ST-061 context-depth-2-includes-shared-adrs
type: session
status: passing
validates:
  features:
  - FT-027
  - FT-043
  adrs:
  - ADR-006
  - ADR-012
  - ADR-018
phase: 1
runner: cargo-test
runner-args: tc_679_session_st_061_context_depth_2_includes_shared_adrs
last-run: 2026-04-22T12:59:08.455929045+00:00
last-run-duration: 0.2s
---

Session ST-061 — --depth 2 reaches ADRs that are only linked transitively through a shared dependency.