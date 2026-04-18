---
id: ADR-040
title: Unified Verify Pipeline and the LLM Boundary
status: accepted
features: []
supersedes: []
superseded-by: []
domains:
- api
- error-handling
- observability
scope: cross-cutting
content-hash: sha256:458d5e6e41e657d611e113d6464b52057b2cdc49153bcf04143cfe3ba709b0cc
---

**Status:** Accepted

**Context:** Product has multiple verification commands that must be run in a specific order to produce a meaningful result. Currently a developer or CI script must know and manually sequence: `product request log verify`, `product graph check`, `product metrics threshold`, `product verify FT-XXX` per feature, `product verify --platform`. Each command has its own exit-code semantics and output format.

There is no single entry point that says "is this repository in a good state?" The closest analogy — `dotnet build` for a solution, `cargo check` for a workspace — is missing.

Simultaneously, several Product commands (`product gap check`, `product drift check`, `product adr review --staged`, `product adr check-conflicts`) internally invoke an LLM. This violates the knowledge boundary established in ADR-021: Product is a knowledge tool. It assembles, validates, and presents information. It does not invoke LLMs. Product assembling a context bundle and piping it to an LLM is orchestration. Product assembling the bundle and writing it to stdout is knowledge provision.

These two concerns are the same architectural decision: what `product verify` runs, and what Product never does. The pipeline that tells you whether the repository is healthy must be deterministic, fast, and reproducible. LLM calls are none of those. The pipeline therefore only runs structural, metric, and test checks. Semantic checks (gap analysis, drift detection, cross-ADR consistency) remain available as `*-bundle` and `*-diff` commands that produce LLM-ready output on stdout for the user to direct as they choose.

**Decision:** `product verify` with no arguments runs a unified six-stage verification pipeline. LLM-dependent checks are removed from every Product command; they become bundle-producing commands (`product gap bundle`, `product drift diff`, `product adr conflict-bundle`) that write LLM-ready input to stdout. Product makes zero LLM API calls in production use.

---

### Pipeline Stages

`product verify` with no arguments runs six stages, ordered by cost and dependency:

```
Stage 1  Log integrity      — product request log verify
Stage 2  Graph structure    — product graph check
Stage 3  Schema validation  — schema-version compat check
Stage 4  Metrics thresholds — product metrics threshold
Stage 5  Feature TCs        — product verify FT-XXX per in-scope feature
Stage 6  Platform TCs       — product verify --platform
```

Each stage runs regardless of whether earlier stages failed — the pipeline always produces a complete picture. The exit code is the worst result across all stages: 0 (all pass), 1 (any E-class error), 2 (warnings only).

| Stage | Pass | Error (exit 1) | Warning (exit 2) | Cost |
|---|---|---|---|---|
| 1 Log integrity | All hashes valid, chain intact | E015 hash mismatch, E016 chain break | W021 tag without log entry | O(N), <1s |
| 2 Graph structure | Zero E-class findings | Any E001–E013 | W-class only | O(V+E), <500ms |
| 3 Schema validation | Schema version compatible | E008 schema ahead of binary | W007 upgrade available | instant |
| 4 Metrics thresholds | All thresholds satisfied | Any severity=error threshold breached | Any severity=warning threshold breached | <200ms |
| 5 Feature TCs | All runnable TCs passing | Any TC failing | Any TC unrunnable (none failing) | dominates wall time |
| 6 Platform TCs | All platform TCs passing | Any platform TC failing | Any platform TC unrunnable | fast + any custom |

Stage 5 runs features reachable from the current phase gate; features in locked phases are skipped with a note. Stage 5 respects the ADR-021 runner boundary: Product only calls the configured runner and reads its exit code.

---

### Scope Flags

```bash
product verify                      # everything — all phases, all features
product verify --phase 1            # scope to phase 1 features only
product verify FT-001               # scope to one feature (unchanged behaviour)
product verify --ci                 # structured JSON output, no colour
```

`product verify FT-XXX` remains the per-feature command. Its behaviour is unchanged: run TCs for that feature, update status, create the git tag on completion. It is also stage 5 of the full pipeline when called without a feature argument.

---

### `--ci` Output

Structured JSON to stdout for pipeline integration:

```json
{
  "passed": false,
  "exit": 1,
  "stages": [
    { "stage": 1, "name": "log-integrity",     "status": "pass",    "findings": [] },
    { "stage": 2, "name": "graph-structure",   "status": "warning", "findings": ["W012", "W016", "W017"] },
    { "stage": 3, "name": "schema-validation", "status": "pass",    "findings": [] },
    { "stage": 4, "name": "metrics",           "status": "warning", "findings": ["bundle_tokens_p95"] },
    { "stage": 5, "name": "feature-tcs",       "status": "fail",    "findings": [
        { "tc": "TC-007", "feature": "FT-003", "status": "failing" }
    ]},
    { "stage": 6, "name": "platform-tcs",      "status": "pass",    "findings": [] }
  ]
}
```

---

### The LLM Boundary

Every Product command that previously invoked an LLM is split into two commands: a structural-only check (fast, deterministic, no LLM) and a bundle-producing command (LLM-ready input to stdout).

| Previous command | Structural-only replacement | Bundle-producing replacement |
|---|---|---|
| `product gap check` (LLM) | `product gap check` (structural heuristics only) | `product gap bundle ADR-XXX` → stdout |
| `product drift check` (LLM) | `product drift check` (file-change detection) | `product drift diff FT-XXX` → stdout |
| `product adr check-conflicts` (LLM) | `product adr check-conflicts` (structural consistency) | `product adr conflict-bundle ADR-XXX` → stdout |
| `product adr review --staged` (structural + LLM) | `product adr review --staged` (structural only) | — |

The bundle commands write a markdown document containing: instructions (which gap/drift/conflict codes to check, what output schema to use), the context bundle (graph state, git diff, or related ADRs), and nothing else. The output is deterministic given the same graph state and git history. The user pipes to the LLM of their choice.

Prompt templates that previously drove internal LLM calls remain in `benchmarks/prompts/` as resources:

```
benchmarks/prompts/
  author-feature-v1.md    # unchanged (authoring)
  author-adr-v1.md        # unchanged (authoring)
  author-review-v1.md     # unchanged (authoring)
  implement-v1.md         # unchanged (implementation context)
  gap-analysis-v1.md      # was internal to product gap check
  drift-analysis-v1.md    # was internal to product drift check
  conflict-check-v1.md    # was internal to adr check-conflicts
```

`product prompts get gap-analysis` prints the content; what the user does with it is their concern.

---

### Updated product.toml

The `[gap-analysis]` section is removed entirely. The `[drift]` section retains `source-roots` and `ignore` (used by `product drift diff` for file discovery) but removes `max-files-per-adr` (no LLM context to cap).

```toml
# REMOVED — Product no longer calls an LLM for gap analysis
# [gap-analysis]
# prompt-version = "1"
# model = "claude-sonnet-4-6"
# max-findings-per-adr = 10
# severity-threshold = "medium"

# BEFORE
# [drift]
# source-roots = ["src/", "lib/"]
# ignore = ["tests/", "benches/", "target/"]
# max-files-per-adr = 20

# AFTER
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
```

---

### Complete LLM Call Inventory After This Change

| Location | Status | Notes |
|---|---|---|
| `product gap check` | **Removed** | Structural checks only |
| `product drift check` | **Removed** | Structural file-change detection only |
| `product adr review --staged` | **Removed** | Structural checks only |
| `product adr check-conflicts` | **Removed** | Structural consistency checks only |
| LLM benchmark (`benchmarks/`) | **Remains** | Not a product feature — self-validation |
| `product gap bundle` | **New** | Produces LLM input, no LLM call |
| `product drift diff` | **New** | Produces LLM input, no LLM call |
| `product adr conflict-bundle` | **New** | Produces LLM input, no LLM call |

Product makes zero LLM calls in production use. All semantic analysis is delegated to the user's toolchain via the `*-bundle` and `*-diff` output commands.

---

### Invariants

⟦Γ:Invariants⟧{
  product_verify_makes_zero_llm_api_calls_at_any_stage
  product_verify_runs_all_six_stages_regardless_of_earlier_failures
  exit_code_is_worst_across_all_stages_0_pass_1_error_2_warning
  structural_commands_complete_under_one_second_on_realistic_repos
  bundle_commands_produce_deterministic_output_from_graph_and_git_history
  prompt_files_are_versioned_resources_never_executed_by_product
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

**Evidence TCs:** TC-552 (ST-110 pipeline all-pass), TC-553 (E-class fail), TC-554 (W-class warn), TC-555 (failing TC fail), TC-556 (locked phase skip), TC-557 (phase scope), TC-558 (CI JSON), TC-559 (feature-scope unchanged), TC-560 (log integrity stage 1), TC-561 (metrics stage 4); TC-566 (gap-check invariant), TC-575 (conflict-check invariant) anchor the zero-LLM-call guarantee.

---

**Rationale:**
- A single-entry verify pipeline mirrors the mental model users already have from `cargo check`, `dotnet build`, and `go vet`. Sequencing six commands manually is a burden; a single command with a clear exit-code contract is CI-native.
- Running every stage regardless of earlier-stage failures produces a complete report in one invocation. Short-circuiting on first failure would force developers to iterate: fix the graph, re-run, discover a metric failure, fix it, re-run. One run, one report, one decision.
- Removing LLM calls from the pipeline is the only way to make it deterministic. LLM output varies run-to-run; a CI gate that varies is unusable. Bundle-producing commands give the user full control over model choice, temperature, determinism strategy, and cost.
- The bundle commands are deterministic because they are functions of the graph state and git history. The same inputs always produce the same bundle. Any LLM non-determinism is the user's concern, not Product's.
- `product gap bundle`, `product drift diff`, and `product adr conflict-bundle` use the same prompt files that previously lived inside Product's binary. Versioning those prompts as repository resources means teams can evolve them without Product releases.
- Retaining structural-only versions of the four LLM-using commands preserves the fast feedback loop. Pre-commit hooks remain useful; they just run faster and no longer cost money per commit.

**Rejected alternatives:**
- **Keep LLM calls inside Product behind a feature flag.** Two code paths, two sets of failure modes, two sets of billing considerations. Rejected: the boundary is a principle, not a toggle.
- **Make `product verify` call LLMs optionally via `--semantic`.** Conflates the pipeline's determinism contract. CI runs with the flag would produce unstable results; users would disable it and forget it exists. Rejected.
- **Short-circuit on first failure.** Faster on failure, but produces an incomplete report. Developers end up re-running the pipeline multiple times. Rejected.
- **Parallel stage execution.** Shorter wall time, but stages have ordering semantics (log integrity before graph structure, metrics after schema) and debugging interleaved output is painful. Rejected for now; revisitable if wall time becomes a bottleneck.
- **Delete the structural `product gap check` entirely and leave only `product gap bundle`.** The structural check is useful — G002 (invariant without TC), G003 (no rejected alternatives), G008 (DEP with no governing ADR) are all mechanically checkable. Rejected: retain deterministic checks, remove only the LLM-dependent checks.

**Test coverage:** TC-552 through TC-562 (verify pipeline) and TC-563 through TC-576 (LLM boundary). See FT-044 and FT-045.
