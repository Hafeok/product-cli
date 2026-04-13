---
id: TC-162
title: FT-020 migration extracts and confirms
type: exit-criteria
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_162_ft_020_migration_extracts_and_confirms"
---

## Description

End-to-end migration test: validates the full two-phase extract-then-confirm workflow from ADR-017.

1. Validate mode (--validate) prints plan without writing files
2. Execute mode (--execute) creates feature files from PRD (excluding non-feature headings), ADR files, and test criteria files
3. Status inference works (checked items → complete, unchecked → planned)
4. Source documents are unchanged after migration
5. Re-running migration skips existing files
6. W008/W009 warnings fire for missing status and missing test sections