---
id: TC-882
title: agent-init writes copilot SKILL.md to .github/skills when target enabled
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
runner-args: tc_882_agent_init_writes_copilot_skill_to_github_skills
---

## Description

When `.product/config.toml` declares `[agent-context.skill]
targets = ["claude", "copilot"]`, running `product agent-init`
writes `SKILL.md` to **both** `.claude/skills/product/SKILL.md`
and `.github/skills/product/SKILL.md` under the repo root. The
file at the Copilot-native path is the same YAML-fronted
`SKILL.md` shape (per the GitHub Copilot "Add skills" docs),
**not** a stripped-down `copilot-instructions.md`.

This TC supersedes the earlier (incorrect) FT-106 draft that
modelled Copilot as a separate plain-markdown renderer at
`.github/copilot-instructions.md`. The corrected design exploits
the shared `SKILL.md` format Copilot and Claude Code both
consume.

**observes:** [file]

## Procedure

1. `Session::new()` — initialise a fresh Product repo in a
   tempdir.
2. Patch `.product/config.toml` to set
   `[agent-context.skill]\ntargets = ["claude", "copilot"]`.
3. Invoke `product agent-init` via `assert_cmd`.
4. Read `<repo>/.github/skills/product/SKILL.md` from disk.
5. Read `<repo>/.claude/skills/product/SKILL.md` from disk.

## Assertions

- The file at `<repo>/.github/skills/product/SKILL.md` exists
  and is non-empty.
- The parent directory `<repo>/.github/skills/product/` was
  created if it did not exist (positive control that the writer
  creates parent dirs).
- The Claude-path artifact at
  `<repo>/.claude/skills/product/SKILL.md` also exists —
  enabling the Copilot target does not suppress the Claude
  target.
- The Copilot-path artifact's first line is `---` (front-matter
  opener), confirming it is the YAML-fronted skill shape and not
  a stripped variant.
- The Copilot-path artifact's front-matter parses and contains
  `name: product` and a non-empty `description:`.
- The command's stdout includes both `Generated:
  ...github/skills/product/SKILL.md` and `Generated:
  ...claude/skills/product/SKILL.md` lines.

The two file existence checks are load-bearing per PAT-003;
asserting only on the stdout would not catch a writer that
silently no-oped.
