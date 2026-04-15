---
id: TC-468
title: adr source files add and remove
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product adr source-files ADR-XXX --add src/drift.rs --add src/drift/`. Assert the `source-files` list in front-matter contains both entries. Run `--remove src/drift.rs`. Assert it is removed and `src/drift/` remains. Run `--add src/nonexistent.rs` for a path that doesn't exist. Assert exit code 0 with a W-class warning (path validated but not required to exist).