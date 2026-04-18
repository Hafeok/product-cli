---
id: TC-470
title: all field mutation tools are idempotent
type: invariant
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_470_all_field_mutation_tools_are_idempotent"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.4s
---

⟦Γ:Invariants⟧{
  ∀tool ∈ {feature_domain, feature_acknowledge, adr_domain, adr_scope, adr_supersede, adr_source_files, test_runner}:
    ∀args:ValidArgs:
      apply(tool, args) ∧ apply(tool, args) = apply(tool, args)
      ∧ file_content(after_first) = file_content(after_second)
}

All field mutation tools are idempotent: calling the same tool with the same arguments twice produces the same file content as calling it once. No duplicates in list fields, no errors on redundant add or remove operations.