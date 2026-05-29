---
id: TC-876
title: agent-init writes skill to project path
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
runner-args: tc_876_agent_init_writes_skill_to_project_path
---

## Description

In a fresh `product init` repo with default config, running
`product agent-init` writes the embedded skill body to
`.claude/skills/product/SKILL.md` under the repo root.

**observes:** [file]

## Procedure

1. `Session::new()` — initialise a fresh Product repo in a tempdir.
2. Invoke `product agent-init` via `assert_cmd`.
3. Read `<repo>/.claude/skills/product/SKILL.md` from disk.

## Assertions

- The file at `<repo>/.claude/skills/product/SKILL.md` exists.
- Its first line is `---` (front-matter opener).
- Its body contains the literal string `name: product` and
  `description:` keys in the front-matter block.
- Its body is byte-identical to the embedded `SKILL.md` source
  shipped with the binary (compare against
  `include_str!("../skills/product-v1.md")`).
- The command's stdout includes a line `Generated: ...SKILL.md`.

The file existence and content assertions are load-bearing per
PAT-003 — checking only the stdout line would reproduce the
FT-046 envelope-only failure mode.
