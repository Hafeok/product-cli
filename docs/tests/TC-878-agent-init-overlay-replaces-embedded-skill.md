---
id: TC-878
title: agent-init overlay replaces embedded skill
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
runner: cargo-test
runner-args: tc_878_agent_init_overlay_replaces_embedded_skill
---

## Description

When `.product/config.toml` declares an `overlay` path under
`[agent-context.skill]` and the file at that path exists, the
overlay contents are installed in place of the embedded default.
This lets a downstream repo customise the skill (e.g. add
repo-specific MCP tool names) without forking the CLI.

**observes:** [file]

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Write a custom skill body to `docs/skills/product.md`
   containing the sentinel string `OVERLAY_SENTINEL_42`.
3. Patch `.product/config.toml` to add
   `[agent-context.skill]\noverlay = "docs/skills/product.md"`.
4. Invoke `product agent-init`.
5. Read `<repo>/.claude/skills/product/SKILL.md` from disk.

## Assertions

- The installed file at
  `<repo>/.claude/skills/product/SKILL.md` exists.
- Its contents are byte-identical to the overlay file (read both,
  compare bytes).
- The contents include `OVERLAY_SENTINEL_42` (positive proof the
  overlay path was followed, not the embedded default).
- The contents do **not** include any string unique to the
  embedded default (e.g. the embedded version banner) — negative
  proof that the embedded body was not silently appended.

Both byte equality and sentinel presence are required: byte
equality alone could pass against a coincidentally-identical
default, and sentinel presence alone could pass if the overlay
were appended to the default.
