---
id: TC-363
title: feature_link_interactive_confirm
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_363_feature_link_interactive_confirm"
last-run: 2026-04-14T18:28:17.026062993+00:00
last-run-duration: 0.2s
---

run `product feature link FT-009 --adr ADR-021`. Assert interactive prompt shows inferred TC links. On confirmation, assert TC links applied atomically with the ADR link.