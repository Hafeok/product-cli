---
id: TC-436
title: init appends to existing .gitignore
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_436_init_appends_to_existing_gitignore"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Create a temporary directory with an existing `.gitignore` containing `target/`. Run `product init --yes`. Assert:

1. `.gitignore` still contains `target/` (original content preserved).
2. `.gitignore` now also contains `docs/graph/`.
3. Running `product init --force --yes` again does not duplicate the `docs/graph/` entry.