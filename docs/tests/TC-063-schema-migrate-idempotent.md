---
id: TC-063
title: schema_migrate_idempotent
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
runner-args: "tc_063_schema_migrate_idempotent"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

run `product migrate schema` twice. Assert the second run reports zero files changed.