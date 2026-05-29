---
id: TC-883
title: all enabled skill targets are byte-identical
type: invariant
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
runner-args: tc_883_all_enabled_skill_targets_are_byte_identical
---

## Description

Invariant: when multiple targets are enabled in
`[agent-context.skill].targets`, every installed `SKILL.md`
file is byte-for-byte identical. There is one canonical body
(embedded or overlay) and `apply_install_skill` copies it
unchanged to each resolved target path — no per-tool
transformation, no per-target header injection.

This invariant is what makes the corrected FT-106 design
honest: if the bytes diverged across targets, the spec's claim
that "Claude and Copilot read the same `SKILL.md` shape" would
silently break. The invariant is also a safety net against
future regressions where a contributor adds a per-target
transformation (well-intentioned but unwanted).

**observes:** [file]

⟦Γ:Invariants⟧{
  ∀repo:Repo, ∀cfg:Config where
      cfg.agent-context.skill.targets ⊇ {"claude", "copilot", "agents"}
      ∧ cfg.agent-context.skill.install = true:
    let r = run("product agent-init", repo) in
    bytes_of(repo/".claude/skills/product/SKILL.md" after r)
      = bytes_of(repo/".github/skills/product/SKILL.md" after r)
      ∧ bytes_of(repo/".github/skills/product/SKILL.md" after r)
        = bytes_of(repo/".agents/skills/product/SKILL.md" after r)
    ∧ run_exit_code(r) = 0
}

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Patch `.product/config.toml` to set
   `[agent-context.skill]\ntargets = ["claude", "copilot",
   "agents"]`.
3. Invoke `product agent-init`.
4. Read all three files into byte buffers `claude`, `copilot`,
   `agents`.

## Assertions

- `claude == copilot` byte-for-byte.
- `copilot == agents` byte-for-byte.
- All three buffers' SHA-256 hash matches the embedded source's
  hash (sanity: confirms we are comparing real content, not
  three empty files).
- All three files' first line is `---` (positive control:
  front-matter present in every copy).

The hash check is the load-bearing surface — comparing only
pairs without anchoring to the embedded source would let a bug
that wrote three identically-empty files pass.
