---
id: TC-813
title: pattern_link_requires_cycle_returns_e003
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_813_pattern_link_requires_cycle_returns_e003
---

## Description

Compose a temp repo with two existing patterns `PAT-001` and
`PAT-002`. Run `product pattern link PAT-001 --requires PAT-002` and
confirm success. Run `product pattern link PAT-002 --requires
PAT-001` and capture (a) the exit code and (b) the stdout/stderr
text.

Assert:

1. The second invocation exits with code 3 (ADR-013 mapping for the
   E003 cycle-detection error, reused from FT-062's depends-on slice).
2. The stderr (or stdout, depending on the existing CLI rendering)
   contains the substring `E003` and the substring `cycle`.
3. The file `docs/patterns/PAT-002-*.md` is unchanged (read the file
   before and after; assert byte equality).

The TC observes `exit-code` (load-bearing) and `stdout` (diagnostic
text). Crucially, it also observes the **unchanged file** — the
guarantee that a refused operation does not partially write.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing patterns PAT-001 and PAT-002, with
  PAT-001 declaring requires: [PAT-002],
When the user runs `product pattern link PAT-002 --requires PAT-001`,
Then the command exits with code 3,
And the stderr or stdout matches /E003/ and /cycle/,
And the on-disk content of `docs/patterns/PAT-002-*.md` is
  byte-identical to its pre-invocation snapshot.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
