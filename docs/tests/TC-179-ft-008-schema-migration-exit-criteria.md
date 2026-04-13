---
id: TC-179
title: ft_008_schema_migration_exit_criteria
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-008
  adrs:
  - ADR-014
phase: 2
---

Run `product migrate schema` on a v0 repository. All files are updated and `schema-version` is bumped. Run two concurrent commands — one succeeds, one exits E010. No data corruption occurs.
