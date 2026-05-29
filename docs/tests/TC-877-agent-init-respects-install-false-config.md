---
id: TC-877
title: agent-init respects install false config
type: scenario
status: unimplemented
validates:
  features:
  - FT-106
  adrs:
  - ADR-031
phase: 6
observes:
- file
- stdout
runner: cargo-test
runner-args: tc_877_agent_init_respects_install_false_config
---

## Description

When `.product/config.toml` contains `[agent-context.skill]
install = false`, running `product agent-init` writes `AGENTS.md`
as usual but does **not** create the skill file. The opt-out is
honoured even on a fresh repo with no prior skill installed.

**observes:** [file, stdout]

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Patch `.product/config.toml` to add
   `[agent-context.skill]\ninstall = false`.
3. Invoke `product agent-init` via `assert_cmd`.

## Assertions

- `<repo>/AGENTS.md` exists (existing behaviour preserved).
- `<repo>/.claude/skills/product/SKILL.md` does **not** exist on
  disk after the run — the assertion reads the parent directory
  and confirms absence rather than trusting the envelope.
- The command's stdout contains the literal `Skipped: skill
  install disabled in config`.
- Exit code is 0 (opt-out is not an error).

The disk-absence check is the load-bearing surface (PAT-003); a
stdout-only check would let a regression that wrote the file
anyway pass.
