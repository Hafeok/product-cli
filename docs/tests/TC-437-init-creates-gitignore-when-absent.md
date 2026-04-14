---
id: TC-437
title: init creates .gitignore when absent
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_437_init_creates_gitignore_when_absent"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Run `product init --yes` in a temporary directory with no `.gitignore`. Assert:

1. `.gitignore` is created.
2. `.gitignore` contains `docs/graph/`.
3. `.gitignore` contains a comment header (`# Product CLI`).