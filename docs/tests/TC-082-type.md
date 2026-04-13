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
---

Migration infers test type from bullet keywords: "chaos" produces type: chaos, "invariant" produces type: invariant, others produce type: scenario.