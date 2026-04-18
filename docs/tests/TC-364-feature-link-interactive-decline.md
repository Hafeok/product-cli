---
id: TC-364
title: feature_link_interactive_decline
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_364_feature_link_interactive_decline"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

decline the interactive TC link prompt. Assert only the ADR link is applied. Assert TC files unchanged.