---
id: TC-468
title: adr source files add and remove
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_468_adr_source_files_add_and_remove"
last-run: 2026-04-15T11:45:30.776758337+00:00
last-run-duration: 0.2s
---

Run `product adr source-files ADR-XXX --add src/drift.rs --add src/drift/`. Assert the `source-files` list in front-matter contains both entries. Run `--remove src/drift.rs`. Assert it is removed and `src/drift/` remains. Run `--add src/nonexistent.rs` for a path that doesn't exist. Assert exit code 0 with a W-class warning (path validated but not required to exist).