---
id: TC-663
title: slice_adapter_structural_invariants
type: invariant
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-043
phase: 3
runner: cargo-test
runner-args: tc_663_slice_adapter_structural_invariants
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.3s
---

## TC — slice + adapter architecture holds (fitness invariant)

ADR-043 pins the vertical-slice + adapter shape. The structural
contracts — no `println!` in slices, `plan_*` pure, `apply_*`
minimal-I/O, `CmdResult` on non-exempt handlers, adapter <400
lines — must hold for the live codebase, not just the ADR
prose. A static fitness test greps the source tree and the AST
for each invariant.

⟦Σ:Types⟧{
  SlicePath≜"src/<slice>/" with tests.rs sibling;
  AdapterPath≜"src/commands/<cmd>.rs";
  Func≜(path: Path, name: String, body: Tokens);
  ReturnType≜"CmdResult" | "BoxResult"
}
⟦Γ:Invariants⟧{
  (A)  ∀f ∈ functions(SlicePath): name(f) matches ^plan_ ⇒
         body(f) contains no println! ∧ no eprintln!
         ∧ no std::process::exit ∧ no std::fs::write
  (B)  ∀f ∈ functions(SlicePath): name(f) matches ^apply_ ⇒
         body(f) uses fileops::write_file_atomic
         ∨ fileops::write_batch_atomic
         ∧ returns Result<_, ProductError>
  (C)  ∀f ∈ functions(SlicePath): name(f) matches ^plan_ ⇒
         return_type(f) is a typed Plan struct (not Result<(), _>
         that forces I/O into the signature)
  (D)  ∀adapter ∈ AdapterPath: line_count(adapter) ≤ 400
  (E)  every adapter whose return type is BoxResult has a
         documented reason in the file's module doc-comment
         matching one of the four retention criteria
         (exit-2 semantics, interactive stdin, continuous progress,
         trivial printer)
  (F)  ∀multi_file_write ∈ SlicePath:
         uses fileops::write_batch_atomic in a single apply_* call
         (no ad-hoc loop of write_file_atomic for cascades)
}
⟦Λ:Scenario⟧{
  given≜the current src/ tree at HEAD
  when≜the fitness test iterates src/feature/, src/adr/,
       src/tc/, src/status/, src/cycle_times/, src/commands/
  then≜every invariant (A)–(F) holds; any violation fails the
       test with a message pointing at the offending file,
       function, and rule name
}
⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩