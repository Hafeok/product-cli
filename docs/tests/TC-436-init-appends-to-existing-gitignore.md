---
id: TC-436
title: init appends to existing .gitignore
type: scenario
status: unimplemented
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_436_init_appends_to_existing_gitignore"
---

## Description

Create a temporary directory with an existing `.gitignore` containing `target/`. Run `product init --yes`. Assert:

1. `.gitignore` still contains `target/` (original content preserved).
2. `.gitignore` now also contains `docs/graph/`.
3. Running `product init --force --yes` again does not duplicate the `docs/graph/` entry.
