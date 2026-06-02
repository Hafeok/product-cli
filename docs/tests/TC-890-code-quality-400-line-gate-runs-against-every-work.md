---
id: TC-890
title: code-quality 400-line gate runs against every workspace member
type: invariant
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-029
phase: 6
observes:
- exit-code
runner: cargo-test
runner-args: tc_890_code_quality_400_line_gate_runs_against_every_workspace_member
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 0.2s
---

## Description

ADR-029 enforces a 400-line hard limit and an SRP doc-comment rule
on every Rust source file under `src/`. Before FT-107, that meant
walking one `src/`. After the split, the same gate must walk every
member crate's `src/` — otherwise a file growing past 400 lines in
`product-mcp` would silently slip past CI.

## Formal

⟦Σ:Types⟧{
Crate ≜ ⟨name: String, src-dir: Path⟩
SourceFile ≜ ⟨path: Path, crate: Crate, line-count: Nat⟩
Workspace ≜ ⟨members: Set[Crate]⟩
collect ≜ Workspace → Set[SourceFile]
LIMIT ≜ 400
}

⟦Γ:Invariants⟧{
∀ ws ∈ Workspace: ∀ f ∈ collect(ws): f.line-count ≤ LIMIT
∀ ws ∈ Workspace: ∀ c ∈ ws.members: ∃ f ∈ collect(ws): f.crate = c
}

The second clause forces collection to visit every member crate
— a workspace-aware gate that silently skipped `product-mcp`
would satisfy the first clause vacuously and miss real violations.

## Procedure

1. The generalised `tests/code_quality_tests.rs` (now living at
   `product-cli/tests/code_quality_tests.rs`) collects every member
   crate's `src/` directory by reading the workspace `Cargo.toml`
   `[workspace] members` list and iterating each member's `src/`.
2. Run the test binary and capture **exit-code**.
3. Inject a temporary 500-line file at `product-mcp/src/__scratch.rs`
   in a sub-test and assert the gate rejects it. Remove the file
   afterwards. (Use a `#[test]` with `tempfile`-style isolation or
   skip the destructive assertion and check the gate's collected
   file list contains a known `product-mcp/src/*.rs` file as a
   proxy.)
4. Run `cargo t --test code_quality_tests` and assert **exit-code**
   is `0` on the post-split tree (every existing file still ≤ 400
   lines).

## Expected

- Step 2 exits `0`.
- Step 3 confirms the gate has visibility into `product-mcp` and
  `product-core` directories — the collected file list contains
  at least one path under each member's `src/`.
- Step 4 exits `0`.

This TC pins the fitness function to the new layout. The
**exit-code** surface is the natural assertion because the gate's
contract is "fail CI if a file is over budget".