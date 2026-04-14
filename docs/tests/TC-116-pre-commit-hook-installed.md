---
id: TC-116
title: pre_commit_hook_installed
type: scenario
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_116_pre_commit_hook_installed"
last-run: 2026-04-14T11:00:34.067694158+00:00
---

run `product install-hooks`. Assert `.git/hooks/pre-commit` exists and is executable.