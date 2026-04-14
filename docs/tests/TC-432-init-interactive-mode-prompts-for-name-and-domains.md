---
id: TC-432
title: init interactive mode prompts for name and domains
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_432_init_interactive_mode_prompts_for_name_and_domains"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Run `product init` (no `--yes`) with stdin providing: a project name, pressing enter to accept default prefixes, and selecting one domain. Assert:

1. The generated `product.toml` contains the provided project name.
2. The selected domain appears in the `[domains]` section.
3. Default prefixes (FT, ADR, TC) are preserved.
4. Exit code is 0.

Note: This test simulates stdin input via `write_all` to the child process stdin pipe.