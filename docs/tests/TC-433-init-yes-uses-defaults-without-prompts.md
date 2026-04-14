---
id: TC-433
title: init --yes uses defaults without prompts
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_433_init_yes_uses_defaults_without_prompts"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Run `product init --yes --name test-project` in an empty directory with stdin closed (no tty). Assert:

1. The command completes without blocking on input.
2. `product.toml` exists with `name = "test-project"`.
3. `[domains]` section is present but empty (no domain entries).
4. `[mcp]` section is present with `write = false` and `port = 7777`.
5. Exit code is 0.