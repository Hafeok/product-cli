---
id: TC-064
title: schema_migrate_preserves_unknown_fields
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
runner-args: "tc_064_schema_migrate_preserves_unknown_fields"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

add a custom field `custom-tag: foo` to a feature file. Run `product migrate schema`. Assert `custom-tag: foo` is still present in the file after migration.