---
id: FT-005
title: Formal Specification
phase: 1
status: complete
depends-on: []
adrs:
- ADR-011
- ADR-015
tests:
- TC-066
- TC-067
- TC-068
- TC-069
- TC-070
- TC-161
domains:
- data-model
- storage
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  data-model: Formal types and invariants describe data constraints but do not define persistent storage schemas. ADR-015 (file write safety) governs the write path; formal blocks are parsed in-memory per ADR-011/ADR-016.
---

⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩
}

⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
       ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

**Invariant example:**
```markdown
---
id: TC-020
title: Betweenness Centrality Always In Range
type: invariant
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-012]
phase: 3
---

---

## Description

FT-005 covers two closely related concerns. First, it specifies the AISP-influenced formal block notation for Test Criterion files (ADR-011): the block types (`⟦Σ:Types⟧`, `⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:ExitCriteria⟧`, `⟦Λ:Benchmark⟧`, `⟦Ε⟧`), the minimal symbol subset in use, and the hybrid file format where YAML front-matter carries graph metadata while the file body carries formal constraints and prose. Second, it specifies the file write safety guarantees that protect all artifact mutations: atomic temp-file-plus-rename writes (ADR-015) and advisory locking on `.product.lock` to serialise concurrent Product processes. These two concerns share this feature because both were introduced as foundational write-path infrastructure in Phase 1 and both are proven by the same set of test criteria (TC-066 through TC-070, TC-161).

## Functional Specification

### Inputs

- **Formal blocks (ADR-011)**: TC file bodies containing zero or more formal block sections delimited by `⟦…⟧{…}` syntax. The grammar is defined in ADR-016. Input is consumed by the `formal` parser module during `parse_tc`.
- **File write operations (ADR-015)**: a file path and complete new content string, provided by any command that mutates artifact files (`feature status`, `adr status`, `test status`, `feature link`, `checklist generate`, `graph rebuild`, `migrate schema`).
- **Lock acquisition (ADR-015)**: the path to `.product.lock` in the repository root, used by all write commands before modifying any file.

### Outputs

- **Formal blocks**: a typed `Vec<FormalBlock>` AST (variants `Types`, `Invariants`, `Scenario`, `ExitCriteria`, `Benchmark`, `Evidence`) stored on the `TestCriterion` struct. Raw block text is preserved verbatim alongside the AST for faithful context bundle output. Validation diagnostics (E001, W004) are emitted for malformed or empty blocks.
- **Atomic writes**: the target file is atomically replaced with the new content. No intermediate state is observable by other processes. If the write fails, the original file is unchanged and no temp file remains.
- **Lock**: an exclusive advisory lock held for the duration of a write command and released on exit (including on signal).

### State

Stateless between invocations. The formal block AST is in-memory only; it is rebuilt from files on every invocation. The advisory lock state is external (the `.product.lock` file on disk); Product detects stale locks by checking whether the recorded PID is still running.

### Behaviour

1. **Formal block parsing** (ADR-011, ADR-016): after splitting a TC file's front-matter and body, the `formal` module scans the body for `⟦`-delimited blocks. Each block is matched against the known block-type vocabulary. A hand-written recursive descent parser constructs the typed AST. The raw text between `{` and `}` is captured verbatim alongside the AST for round-trip output. Subsequent blocks continue to parse even if one block fails.
2. **Block validation**: `invariant` and `chaos` type TCs without a `⟦Γ:Invariants⟧` block generate W004. Evidence blocks with `δ` outside [0.0, 1.0] or `φ` outside [0, 100] generate E001. Empty block bodies (`⟦Γ:Invariants⟧{}`) generate W004.
3. **Atomic writes** (ADR-015): every `fileops::write_file_atomic` call writes content to a temp file `.<filename>.product-tmp.<pid>` in the same directory, calls `fsync`, then renames atomically to the target path. On failure before the rename, the temp file is deleted.
4. **Startup temp-file cleanup** (ADR-015): on every Product invocation, the configured directories are scanned for `*.product-tmp.*` files left by previously crashed processes and deleted before any command logic runs.
5. **Advisory locking** (ADR-015): write commands acquire an exclusive lock on `.product.lock` with a 3-second timeout using the `fd-lock` crate. If the PID recorded in the lock file is not running (stale lock), the lock is acquired without waiting. The lock is held as a RAII guard and released on process exit.

### Invariants

- The target file is either fully replaced with the new content or unchanged after a write attempt; partial writes cannot occur.
- No `.product-tmp.*` files remain in the repository directories after a Product invocation (either the write succeeded and the temp was renamed, or the write failed and the temp was deleted; startup cleanup removes any residual).
- At most one Product process holds the write lock on a given repository at any time.
- Formal block raw text round-trips faithfully: the bytes between `{` and `}` in the source file appear identically in the context bundle output.
- Evidence block `δ` is in [0.0, 1.0] and `φ` is in [0, 100]; values outside these ranges are hard parse errors (E001), not warnings.

### Error handling

- Write failure before rename (disk full, permission error) → temp file is deleted; original file is unchanged; error reported on stderr with file path.
- Lock not acquired within 3 seconds → E010 with the PID and start time of the lock holder; command exits without modifying any file.
- Stale lock file (holding PID not running) → lock is silently acquired; no error or warning.
- Formal block with unclosed `⟦` delimiter → E001 at the line of the opening delimiter; remaining blocks in the file are still parsed.
- Unrecognised block type → E001 with the unrecognised type name; the block is skipped.
- Empty block body → W004.
- Evidence field out of range → E001.

### Boundaries

- Formal block parsing applies only to Test Criterion files; Feature and ADR files do not contain formal blocks.
- The formal block grammar (ADR-016) is the authority on what constitutes a valid block. FT-005 covers the runtime parse behaviour; the grammar definition itself lives in ADR-016.
- Advisory locking serialises concurrent Product write commands. It does not prevent external editors, git operations, or other tools from modifying artifact files concurrently — the lock is advisory, not mandatory (ADR-015).
- Atomic writes use POSIX rename semantics; Windows rename semantics differ and would require a platform-specific implementation if Windows support is added (ADR-015).

## Out of scope

- Full formal semantic verification of invariant expressions — the parser validates syntax and structure but does not evaluate or prove the logical content of formal blocks.
- The formal block grammar specification itself — defined in ADR-016 and reproduced in the `formal` module source.
- Execution of TC runners (`cargo-test`, `bash`, `pytest`) — covered by the verify pipeline.
- The `benchmark` TC sub-map semantics and execution — the benchmark runner is external to Product; Product only stores and round-trips the `benchmark` front-matter fields.
