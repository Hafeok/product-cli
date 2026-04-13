---
id: TC-031
title: abandon_feature_orphans_tests
type: scenario
status: passing
validates:
  features:
  - FT-018
  adrs:
  - ADR-010
phase: 1
runner: cargo-test
runner-args: "tc_031_abandon_feature_orphans_tests"
---

create FT-001 linked to TC-001 and TC-002. Set FT-001 to `abandoned`. Assert TC-001 and TC-002 have FT-001 removed from their `validates.features`. Assert both tests appear in `product test untested`.