---
id: TC-879
title: agent-init skill target user expands home
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
runner-args: tc_879_agent_init_skill_target_user_expands_home
---

## Description

When `[agent-context.skill] target = "user"` is set (or
`product agent-init --skill-target user` is passed on the CLI),
the skill is installed under the user-global Claude Code skill
directory rather than the project-local `.claude/`. The home
directory is resolved through the same `dirs::home_dir()` path
used elsewhere in the CLI.

**observes:** [file]

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Override the `HOME` environment variable to point at the
   tempdir (so the test never touches the real home).
3. Invoke `product agent-init --skill-target user` with the
   patched `HOME`.
4. Read `<HOME>/.claude/skills/product/SKILL.md` from disk.

## Assertions

- The file at `<HOME>/.claude/skills/product/SKILL.md` exists
  and is non-empty.
- `<repo>/.claude/skills/product/SKILL.md` does **not** exist —
  the `--skill-target user` flag suppresses the project-local
  write.
- The user-global file's contents are byte-identical to the
  embedded default (this test does not exercise the overlay
  path; that's TC-878's job).

Both files are inspected directly on disk; the test does not rely
on the command's stdout to determine which path was written
(PAT-003).
