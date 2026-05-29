---
id: TC-884
title: targets list controls which paths receive the skill
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
runner-args: tc_884_targets_list_controls_which_paths_receive_the_skill
---

## Description

The `targets` list in `[agent-context.skill]` is the per-path
opt-in. Names present in the list are written; names absent are
not. This TC exercises three configurations to lock the contract:

- `targets = ["claude"]` — default; only the Claude path is
  written. The Copilot and agents paths must not appear on
  disk.
- `targets = ["copilot"]` — only the Copilot path is written;
  the Claude path is silent. This is the configuration a
  Copilot-only repo would use.
- `targets = []` — empty list; no skill files are written, but
  the master `install` switch is still `true`. This is the
  fine-grained way to disable all skill writes without flipping
  the master switch.

**observes:** [file, stdout]

## Procedure

For each of the three configurations above, in a fresh
`Session::new()` tempdir:

1. Patch `.product/config.toml` to the configured
   `[agent-context.skill]\ntargets = [...]` value.
2. Invoke `product agent-init` via `assert_cmd`.
3. Read the existence (or absence) of each of:
   - `<repo>/.claude/skills/product/SKILL.md`
   - `<repo>/.github/skills/product/SKILL.md`
   - `<repo>/.agents/skills/product/SKILL.md`
4. Capture stdout.

## Assertions

Configuration 1 (`targets = ["claude"]`):

- `<repo>/.claude/skills/product/SKILL.md` exists.
- `<repo>/.github/skills/product/SKILL.md` does **not** exist —
  verified by reading the parent directory.
- `<repo>/.agents/skills/product/SKILL.md` does **not** exist.
- stdout contains exactly one `Generated: ...SKILL.md` line.

Configuration 2 (`targets = ["copilot"]`):

- `<repo>/.github/skills/product/SKILL.md` exists.
- `<repo>/.claude/skills/product/SKILL.md` does **not** exist.
- `<repo>/.agents/skills/product/SKILL.md` does **not** exist.
- stdout contains exactly one `Generated: ...SKILL.md` line.

Configuration 3 (`targets = []`):

- None of the three skill files exist on disk.
- `<repo>/AGENTS.md` still exists (existing behaviour
  preserved).
- stdout does **not** contain a `Generated: ...SKILL.md` line.
- Exit code is 0 (empty `targets` is a valid configuration).

The disk-state assertions across all three configurations are
load-bearing per PAT-003 — checking only stdout would allow a
regression that wrote unwanted files to pass silently.
