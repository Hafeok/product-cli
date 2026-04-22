---
id: FT-015
title: Test Criteria
phase: 1
status: complete
depends-on: []
adrs:
- ADR-011
- ADR-016
- ADR-018
tests:
- TC-035
- TC-036
- TC-037
- TC-038
- TC-039
- TC-040
- TC-071
- TC-072
- TC-073
- TC-074
- TC-075
- TC-076
- TC-077
- TC-078
- TC-079
- TC-153
domains:
- data-model
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

### TC-001 — Binary Compiles (exit-criteria)

[prose description]

⟦Λ:ExitCriteria⟧{
  binary_size < 20MB
  compile_time(rpi5, cold) < 5min
  ldd(binary) = {libc}
}
⟦Ε⟧⟨δ≜0.98;φ≜100;τ≜◊⁺⟩

### TC-002 — Raft Leader Election (scenario)

[prose description]

⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower|Learner }
⟦Γ:Invariants⟧{ ∀s:ClusterState: |{n | roles(n)=Leader}| = 1 }
⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

The bundle evidence block `⟦Ε⟧` at the top is computed as the mean of all linked test criterion `δ` values (confidence), and the percentage of criteria with formal blocks present (`φ`). An agent receiving this bundle can assess the specification quality before reading the full content.

YAML front-matter is stripped from all sections. Formal blocks in test criteria are preserved verbatim — they are the specification, not metadata.

---