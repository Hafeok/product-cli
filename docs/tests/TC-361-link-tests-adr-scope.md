---
id: TC-361
title: link_tests_adr_scope
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_361_link_tests_adr_scope"
last-run: 2026-04-18T10:41:58.940827843+00:00
last-run-duration: 0.2s
---

run `product migrate link-tests --adr ADR-002`. Assert only TCs linked to ADR-002 are updated. TCs for ADR-006 unchanged.