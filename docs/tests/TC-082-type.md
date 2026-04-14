---
id: TC-082
title: type
type: scenario
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_082_type"
last-run: 2026-04-14T14:25:40.415822949+00:00
---

Migration infers test type from bullet keywords: "chaos" produces type: chaos, "invariant" produces type: invariant, others produce type: scenario.