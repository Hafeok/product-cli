---
id: TC-083
title: status
type: scenario
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_083_status"
last-run: 2026-04-14T14:25:40.415822949+00:00
---

Migration extracts ADR status from **Status:** lines. Accepted, Proposed, Superseded, Abandoned are recognized. Missing status defaults to proposed with W008 warning.