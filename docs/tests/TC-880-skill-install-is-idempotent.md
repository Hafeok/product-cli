---
id: TC-880
title: skill install is idempotent
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
runner-args: tc_880_skill_install_is_idempotent
---

## Description

Invariant: running `product agent-init` N times in a row produces
the same `.claude/skills/product/SKILL.md` byte-for-byte. No
timestamps, generation counters, or order-dependent fields leak
into the skill body. This is the safety net against accidental
non-determinism that would force the file into the diff of every
re-run.

**observes:** [file]

⟦Γ:Invariants⟧{
  ∀repo:Repo, ∀n:Nat where n ≥ 2:
    let r₁ = run("product agent-init", repo) in
    let r₂ = run("product agent-init", repo) in
    bytes_of(repo/".claude/skills/product/SKILL.md" after r₁)
      = bytes_of(repo/".claude/skills/product/SKILL.md" after r₂)
    ∧ run_exit_code(r₁) = 0 ∧ run_exit_code(r₂) = 0
}

## Procedure

1. `Session::new()` — initialise a fresh Product repo.
2. Run `product agent-init` once; read the resulting skill file's
   bytes into `first`.
3. Mutate unrelated graph state (e.g. add a feature) so that
   `AGENTS.md` would change on the next run.
4. Run `product agent-init` a second time; read the resulting
   skill file's bytes into `second`.

## Assertions

- `first == second` byte-for-byte.
- The hash of `first` matches the hash of the embedded source
  (sanity: confirms we are comparing real installed content, not
  two empty files).
- The mtime of the skill file did change on the second run (the
  atomic write replaces the file unconditionally) — this is a
  positive control that the second run actually executed.

The byte-equality check is the load-bearing surface. The mtime
check guards against a false positive where the second `agent-init`
silently no-oped and left the first file untouched.
