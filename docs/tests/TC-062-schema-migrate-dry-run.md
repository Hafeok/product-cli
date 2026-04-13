---
id: TC-062
title: schema_migrate_dry_run
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
runner: cargo-test
runner-args: "tc_062_schema_migrate_dry_run"
---

run `product migrate schema --dry-run` on a v1 repo. Assert no files are modified. Assert stdout describes what would change.