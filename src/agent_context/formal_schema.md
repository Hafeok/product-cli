AISP (AI Symbolic Protocol) formal blocks carry typed assertions that
drive Product's validation mechanics for `invariant`, `chaos`, and
`exit-criteria` TCs. Every block is wrapped in the Unicode white square
brackets `⟦` / `⟧` with a type label inside; the block body goes between
`{` and `}` (the Epsilon evidence block uses `⟨` / `⟩` instead).

The five block type labels below are the parser-accepted spellings. See
`src/formal/parser.rs::parse_formal_blocks_with_diagnostics` for the
authoritative list and ADR-016 for the full grammar.

### Sigma-Types — `⟦Σ:Types⟧`

Declare named domain types used by invariants and scenarios.

```
⟦Σ:Types⟧{
  Node ≜ IRI
  Role ≜ Leader | Follower | Learner
  ClusterState ≜ ⟨nodes:Node+, roles:Node→Role⟩
}
```

Required by: `invariant` and `chaos` TCs (W004 — either this block or
`⟦Γ:Invariants⟧` satisfies the formal-block requirement).

### Gamma-Invariants — `⟦Γ:Invariants⟧`

Declare formal assertions that must hold.

```
⟦Γ:Invariants⟧{
  ∀s:ClusterState: |{n ∈ s.nodes | s.roles(n) = Leader}| = 1
}
```

Required by: `invariant` and `chaos` TCs (W004). Also satisfies G002 — an
ADR with `⟦Γ:Invariants⟧` must have a linked TC of type `scenario`, `chaos`,
or `invariant` addressing the invariant.

### Lambda-Scenario — `⟦Λ:Scenario⟧`

Given / when / then structured test flow.

```
⟦Λ:Scenario⟧{
  given ≜ cluster_init(nodes:2)
  when  ≜ elapsed(10s)
  then  ≜ ∃n ∈ nodes: roles(n) = Leader
}
```

Required by: `scenario` TCs (optional but encouraged). Also an acceptable
formal block for `chaos` TCs under W004.

### Lambda-ExitCriteria — `⟦Λ:ExitCriteria⟧`

Measurable thresholds that gate phase completion.

```
⟦Λ:ExitCriteria⟧{
  leader_election_time < 10s
  failover_success_rate ≥ 0.99
}
```

Required by: `exit-criteria` TCs.

### Epsilon (Evidence) — `⟦Ε⟧⟨…⟩`

Specification confidence, coverage, and stability. Fields are
semicolon-separated and enclosed in `⟨` / `⟩` rather than `{` / `}`.

```
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

Fields:

- `δ` — specification confidence, range `[0.0, 1.0]`
- `φ` — coverage completeness, range `[0, 100]`
- `τ` — stability signal: `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown)

Required by: none directly, but encouraged on every `invariant`, `chaos`,
and `exit-criteria` TC. W006 fires when `δ < 0.7`.

### `tc-type` to required block summary

| `tc-type` | Satisfies W004 with | Notes |
|---|---|---|
| `invariant` | `⟦Σ:Types⟧` or `⟦Γ:Invariants⟧` | At least one formal block must be present. |
| `chaos` | `⟦Γ:Invariants⟧` or `⟦Λ:Scenario⟧` | Invariants preferred; scenario acceptable. |
| `exit-criteria` | `⟦Λ:ExitCriteria⟧` | Measurable thresholds that close the phase gate. |
| `scenario` | — | Optional `⟦Λ:Scenario⟧` encouraged. |
| `benchmark` | — | No formal block required. |
| `absence` | — | No formal block required; runner asserts the negative. |
