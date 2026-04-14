---
id: TC-318
title: pre_commit_hook_installed
type: scenario
status: unimplemented
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
---

run `product install-hooks`. Assert `.git/hooks/pre-commit` exists and is executable.