---
id: TC-356
title: link_tests_basic
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_356_link_tests_basic"
last-run: 2026-04-14T18:29:03.925615608+00:00
last-run-duration: 0.2s
---

FT-001 links ADR-002. TC-002 validates ADR-002. Run `product migrate link-tests`. Assert TC-002 gains `validates.features: [FT-001]`. Assert FT-001 gains `tests: [TC-002]`.