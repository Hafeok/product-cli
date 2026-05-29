---
id: TC-881
title: installed skill has valid frontmatter
type: exit-criteria
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
runner-args: tc_881_installed_skill_has_valid_frontmatter
---

## Description

Exit criterion: the installed `SKILL.md` parses as a Claude Code
skill — it has a YAML front-matter block with both `name` and
`description` keys, the body after the front-matter is non-empty,
and the `name` value is exactly `product` so the skill registers
under the expected slot.

This is the contract Claude Code (and any downstream skill
discovery layer) relies on. A skill file that ships without
`name`/`description` is silently ignored at discovery time,
which would reproduce the FT-046 failure mode at the
distribution layer.

**observes:** [file]

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Invoke `product agent-init`.
3. Parse `<repo>/.claude/skills/product/SKILL.md` with the same
   YAML front-matter parser used elsewhere in the codebase
   (`product_lib::parser::parse_front_matter` or equivalent).

## Assertions

- The file's front-matter block parses without error.
- The parsed front-matter contains a `name` field with value
  `product`.
- The parsed front-matter contains a `description` field whose
  value is a non-empty string.
- The body after the front-matter is at least 100 bytes (sanity
  check against an accidentally-truncated install).

All assertions read the file off disk and decode it; none rely
on the `agent-init` command's stdout (PAT-003).
