---
id: TC-358
title: link_tests_cross_cutting_excluded
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_358_link_tests_cross_cutting_excluded"
last-run: 2026-04-14T18:29:03.925615608+00:00
last-run-duration: 0.2s
---

ADR-001 is cross-cutting. TC-001 validates ADR-001. All features link ADR-001. Run `link-tests`. Assert TC-001.validates.features remains empty.