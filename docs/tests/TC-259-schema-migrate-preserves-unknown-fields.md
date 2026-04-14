---
id: TC-259
title: schema_migrate_preserves_unknown_fields
type: scenario
status: unimplemented
validates:
  features: 
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
---

add a custom field `custom-tag: foo` to a feature file. Run `product migrate schema`. Assert `custom-tag: foo` is still present in the file after migration.