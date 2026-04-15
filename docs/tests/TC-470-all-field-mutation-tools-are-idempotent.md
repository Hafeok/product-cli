---
id: TC-470
title: all field mutation tools are idempotent
type: invariant
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

⟦Γ:Invariants⟧{
  ∀tool ∈ {feature_domain, feature_acknowledge, adr_domain, adr_scope, adr_supersede, adr_source_files, test_runner}:
    ∀args:ValidArgs:
      apply(tool, args) ∧ apply(tool, args) = apply(tool, args)
      ∧ file_content(after_first) = file_content(after_second)
}

All field mutation tools are idempotent: calling the same tool with the same arguments twice produces the same file content as calling it once. No duplicates in list fields, no errors on redundant add or remove operations.