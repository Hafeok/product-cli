---
id: ADR-018
title: Testing Strategy — Property-Based, Integration, and LLM Benchmark
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:660cd17f84e0773b44d486846c5fc60addb23fb787907bfad77fcf5a2d9b2c3a
---

**Status:** Accepted

**Context:** Product has three distinct failure classes that require three distinct testing approaches:

1. **Algorithmic correctness** — graph algorithms (topological sort, betweenness centrality, BFS, reachability) and the front-matter parser must produce correct results for all valid inputs, not just the ones the test author thought to write. Unit tests on hand-crafted inputs cannot cover the boundary cases that distributed systems and parser edge cases produce.

2. **Command correctness** — the full CLI surface (argument parsing, file I/O, error formatting, exit codes, stdout/stderr separation) must behave correctly on real repository state. Algorithmic unit tests cannot catch bugs in how the CLI routes a subcommand, formats a diagnostic message, or handles a concurrent write.

3. **Value delivery** — the core claim of Product is that context bundles improve LLM implementation quality. This claim is currently unvalidated. If context bundles do not measurably improve agent outputs, the product's fundamental design assumption is wrong and must be revised.

No single testing approach covers all three. This ADR specifies all three, defines their scope boundaries, and assigns them to phases.

---

### Design 1: Property-Based Testing (proptest)

**Target failure class:** Algorithmic correctness — inputs the test author did not anticipate.

**Tool:** `proptest` crate. Generates thousands of random inputs satisfying user-defined strategies, shrinks failing inputs to minimal reproducible examples.

**Scope:** Pure functions only — graph construction, traversal algorithms, front-matter parser, file write logic. No filesystem, no CLI, no network.

**Repository location:** `tests/property/` — separate from unit tests to allow independent execution and longer run budgets.

#### Generators

```rust
/// Generates a valid DAG of Feature nodes.
/// Only adds edges from lower-index to higher-index nodes,
/// guaranteeing acyclicity by construction.
fn arb_dag(
    size: impl Strategy<Value = usize>,
    edge_density: f64,
) -> impl Strategy<Value = FeatureGraph>

/// Generates a connected graph — required for centrality to be meaningful.
fn arb_connected_graph(
    size: impl Strategy<Value = usize>,
    density: f64,
) -> impl Strategy<Value = FeatureGraph>

/// Generates syntactically valid Feature structs.
/// IDs are valid format, statuses are valid enum values,
/// phases are in 1..=10. Does NOT generate broken links.
fn arb_valid_feature() -> impl Strategy<Value = Feature>

/// Generates arbitrary byte strings including edge cases:
/// empty string, valid UTF-8, invalid UTF-8, lone delimiters,
/// extremely long strings, YAML injection attempts.
fn arb_arbitrary_input() -> impl Strategy<Value = String>

/// Generates a valid YAML key-value pair not in the Product schema.
fn arb_unknown_field() -> impl Strategy<Value = (String, String)>
```

#### Property Set

**Parser robustness (from ADR-013):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P001 | No input causes a panic | `∀s:String: parse_frontmatter(s) ≠ panic` |
| TC-P002 | Valid front-matter round-trips | `∀f:Feature: parse(serialise(f)) = f` |
| TC-P003 | Unknown fields preserved on write | `∀f:Feature, k:UnknownField: serialise(inject(f,k)) ⊇ k` |
| TC-P004 | Malformed input returns structured error | `∀s:InvalidYAML: parse(s) = Err(E001)` |

**Graph algorithm correctness (from ADR-012):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P005 | Topo order respects all dependency edges | `∀g:DAG, (u,v)∈g.edges: pos(topo(g),u) < pos(topo(g),v)` |
| TC-P006 | Topo sort detects all cycles | `∀g:CyclicGraph: topo_sort(g) = Err(E003)` |
| TC-P007 | Centrality always in range | `∀g:ConnectedGraph, n∈g.nodes: 0.0 ≤ centrality(g,n) ≤ 1.0` |
| TC-P008 | Reverse reachability inverts forward | `∀g:Graph, u,v∈g.nodes: reachable(g,u,v) ↔ reachable(rev(g),v,u)` |
| TC-P009 | BFS deduplication — node appears once | `∀g:Graph, seed:Node, d:Depth: |{n \| n∈bfs(g,seed,d)}| = |bfs(g,seed,d)|` |

**File write safety (from ADR-015):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P010 | Atomic write — no torn state | `∀content:String, cutAt:Offset: file_after_interrupt(cutAt) ∈ {original, new}` |
| TC-P011 | Write + re-read is identity | `∀content:String: read(atomic_write(path, content)) = content` |

**Configuration:**

```toml
# .proptest-regressions are committed — shrunk failing cases are permanent regression tests
[proptest]
cases = 1000          # default per property
max_shrink_iters = 500
failure_persistence = "file"   # .proptest-failures/
```

---

### Design 2: Integration Test Harness

**Target failure class:** Command correctness — full CLI behaviour on real repository state.

**Scope:** Full binary execution. Every test runs the compiled `product` binary against a real temporary directory. No mocking.

**Repository location:** `tests/integration/`

#### Harness API

```rust
pub struct Harness {
    dir: TempDir,
    bin: PathBuf,    // path to compiled product binary
}

impl Harness {
    pub fn new() -> Self
    pub fn write(&self, path: &str, content: &str) -> &Self
    pub fn run(&self, args: &[&str]) -> Output
    pub fn read(&self, path: &str) -> String
    pub fn exists(&self, path: &str) -> bool
    pub fn file_mtime(&self, path: &str) -> SystemTime
}

pub struct Output {
    pub stdout:    String,
    pub stderr:    String,
    pub exit_code: i32,
}

impl Output {
    pub fn assert_exit(&self, code: i32) -> &Self
    pub fn assert_stderr_contains(&self, s: &str) -> &Self
    pub fn assert_stderr_matches_error(&self, code: &str) -> &Self
    pub fn assert_stdout_clean(&self) -> &Self   // no YAML, no front-matter
    pub fn assert_json_stderr(&self) -> Value    // parse and return
    pub fn assert_no_tmp_files(&self) -> &Self
}
```

#### Fixture Library

Standard repository configurations defined once, composed freely:

```rust
pub fn fixture_minimal() -> Harness           // 1 feature, 1 ADR, linked
pub fn fixture_broken_link() -> Harness       // feature references non-existent ADR
pub fn fixture_dep_cycle() -> Harness         // FT-001 ↔ FT-002 cycle
pub fn fixture_supersession_cycle() -> Harness // ADR-001 ↔ ADR-002 cycle
pub fn fixture_orphaned_adr() -> Harness      // ADR with no feature links
pub fn fixture_phase_1_complete() -> Harness  // all phase-1 features complete
pub fn fixture_full_picloud() -> Harness      // migrated PiCloud repo (generated once, committed)
```

#### Scenario Test Set

**Error model (ADR-013):**

| TC | Fixture | Command | Asserts |
|---|---|---|---|
| IT-001 | broken_link | `graph check` | exit 1, E002 on stderr, file+line, hint |
| IT-002 | broken_link | `graph check --format json` | exit 1, valid JSON, `errors[0].code="E002"` |
| IT-003 | minimal | `graph check` | exit 0, stdout empty |
| IT-004 | orphaned_adr | `graph check` | exit 2, W001 on stderr |
| IT-005 | minimal | `context FT-001` | exit 0, stdout contains `⟦Ω:Bundle⟧`, no `---` delimiters |
| IT-006 | minimal | `context FT-001 > file` | file created, stderr empty |
| IT-007 | dep_cycle | `graph check` | exit 1, E003 names both features |
| IT-008 | any | bad YAML in feature file | exit 1, E001, no panic |

**Concurrent writes (ADR-015):**

| TC | Setup | Asserts |
|---|---|---|
| IT-009 | Two threads call `feature status FT-001` simultaneously | Exactly one exits 0, one exits 1 (E010). File valid. |
| IT-10 | Stale `.product.lock` (dead PID) | Next write command succeeds, lock cleared |
| IT-11 | Write interrupted at byte N (simulated) | File is either original or new content — never partial |

**Schema versioning (ADR-014):**

| TC | Setup | Asserts |
|---|---|---|
| IT-12 | `schema-version = "99"` | exit 1, E008, upgrade hint |
| IT-13 | `schema-version = "0"` (old) | exit 0, W007 on stderr, command completes |
| IT-14 | `migrate schema --dry-run` on old repo | exit 0, no files changed, stdout describes plan |
| IT-15 | `migrate schema` twice | Second run: exit 0, "0 files changed" |

**Migration (ADR-017):**

| TC | Source | Asserts |
|---|---|---|
| IT-16 | picloud-prd.md `--validate` | exit 0, zero files created, stdout shows plan |
| IT-17 | picloud-adrs.md `--execute` | exit 0, ≥9 ADR files, ≥30 TC files created |
| IT-18 | picloud-prd.md source unchanged | source file byte-identical before/after |
| IT-19 | picloud-prd.md then `graph check` | exit 2 (warnings only, no broken links) |

#### Golden File Tests

Migration output is verified against committed golden files. Intentional heuristic changes require `UPDATE_GOLDEN=1 cargo test` and a reviewed diff.

```
tests/
  fixtures/
    picloud-prd.md
    picloud-adrs.md
  golden/
    features/
      FT-001-cluster-foundation.md
      ...
    adrs/
      ADR-001-rust-language.md
      ...
    tests/
      TC-001-binary-compiles.md
      ...
```

---

### Design 3: LLM Context Quality Benchmark

**Target failure class:** Value delivery — does Product actually improve LLM implementation quality?

**Scope:** End-to-end quality measurement. Runs the compiled binary to generate context bundles, sends them to an LLM, scores the output against a rubric using a separate LLM call.

**Repository location:** `benchmarks/`

**Run cadence:** Not in CI. Triggered manually on release candidates, after context bundle format changes (ADR-006, ADR-011, ADR-012), and monthly for trend tracking.

#### Repository Layout

```
benchmarks/
  runner/
    src/main.rs          ← benchmark runner binary
  tasks/
    task-001-raft-leader-election/
      prompt.md
      rubric.md
    task-002-frontmatter-parser/
      prompt.md
      rubric.md
    task-003-context-bundle-assembly/
      prompt.md
      rubric.md
  results/
    2026-04-11/
      results.json
      results.md         ← human-readable summary
    latest -> 2026-04-11/
```

#### Task Structure

Each task defines a realistic implementation request grounded in the PiCloud project:

**`prompt.md`** — the implementation request, stated without embedded context:
```markdown
Implement the Raft leader election logic for PiCloud's cluster foundation.
The implementation must satisfy the platform's architectural constraints
and pass the defined test criteria for this feature.
```

**`rubric.md`** — binary-scored criteria only. No holistic judgments.
```markdown
# Rubric: Raft Leader Election

## Correctness (weight: 3)
- Uses openraft crate, not a custom Raft implementation
- Implements RaftStorage trait
- Leader election completes within 10s timeout
- Exactly one node holds Leader role at any time
- RDF graph reflects leader identity via picloud:hasRole

## Architecture (weight: 2)
- Implementation is in Rust
- No unwrap() calls on production paths
- No unsafe blocks outside marked modules

## Test coverage (weight: 2)
- Includes a scenario test for leader election
- Includes a chaos test for leader failover
- Invariant checked at test boundaries
```

#### Three Conditions

Every task is run under three context conditions:

| Condition | Context provided |
|---|---|
| `none` | No context beyond the prompt |
| `naive` | Full `picloud-prd.md` + `picloud-adrs.md` concatenated |
| `product` | Output of `product context FT-001 --depth 2` |

#### Scoring Protocol

Each rubric criterion is scored by a separate LLM call with a narrow binary question to minimise scorer variance:

```
"Does the following implementation satisfy this criterion:
'Uses openraft crate, not a custom Raft implementation'?
Answer only YES or NO.

Implementation:
[implementation text]"
```

Final score = Σ(satisfied_criteria × weight) / Σ(all_criteria × weight)

Each condition is run N=5 times at temperature=0. The reported score is the mean across runs.

#### Pass Thresholds

A benchmark TC passes when:
- `score(product) ≥ 0.80` — absolute quality threshold
- `score(product) - score(naive) ≥ 0.15` — Product must add measurable value over naive context

Both conditions must hold. A high product score on an easy task where naive also scores high does not constitute a pass.

#### Result Format

```json
{
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "model": "claude-sonnet-4-6",
  "runs_per_condition": 5,
  "tasks": [
    {
      "id": "task-001-raft-leader-election",
      "tc": "TC-030",
      "conditions": {
        "none":    { "mean": 0.41, "scores": [0.39, 0.41, 0.44, 0.40, 0.41] },
        "naive":   { "mean": 0.63, "scores": [0.61, 0.65, 0.63, 0.62, 0.64] },
        "product": { "mean": 0.87, "scores": [0.85, 0.89, 0.86, 0.88, 0.87] }
      },
      "delta_product_vs_naive": 0.24,
      "pass": true
    }
  ],
  "aggregate": {
    "mean_product_score": 0.87,
    "mean_delta_vs_naive": 0.21,
    "tasks_passing": 3,
    "tasks_total": 3
  }
}
```

Results are committed to the repository. The trend across runs is the primary signal — a declining `delta_product_vs_naive` over releases indicates context bundle quality is regressing.

#### Initial Task Set (Phase 3)

Three tasks covering the three most important features:

| TC | Task | Feature | Key rubric dimension |
|---|---|---|---|
| TC-030 | Raft leader election | FT-001 | Architecture compliance (openraft, RDF) |
| TC-031 | Front-matter parser | FT-Product-001 | Robustness (no panics, error codes) |
| TC-032 | Context bundle assembly | FT-Product-002 | Correctness (depth, dedup, ordering) |

---

### Testing Phase Assignment

| Design | Phase | Prerequisite |
|---|---|---|
| Integration harness infrastructure | Phase 1 | Binary compiles |
| Integration: error model tests (IT-001–IT-008) | Phase 1 | `graph check` implemented |
| Integration: concurrency tests (IT-009–IT-11) | Phase 1 | Write commands implemented |
| Property: parser robustness (TC-P001–TC-P004) | Phase 1 | Parser implemented |
| Integration: schema tests (IT-12–IT-15) | Phase 2 | Schema versioning implemented |
| Integration: migration tests (IT-16–IT-19) | Phase 2 | Migration implemented |
| Property: graph algorithms (TC-P005–TC-P009) | Phase 2 | Algorithms implemented |
| Property: file safety (TC-P010–TC-P011) | Phase 2 | Atomic writes implemented |
| LLM benchmark infrastructure | Phase 3 | Context bundles complete |
| LLM benchmark tasks (TC-030–TC-032) | Phase 3 | Full feature set complete |

---

**Rationale:**
- Three separate designs are necessary because each catches a disjoint failure class. Collapsing them into one approach (e.g., "just write more unit tests") would leave two failure classes untested. The cost of three approaches is justified by the risk distribution.
- Property-based tests are assigned to pure functions only. Attempting to property-test the full CLI (generating random repository structures and asserting on binary output) produces tests that are slow, brittle, and produce unhelpful failure messages. The integration harness handles that scope.
- The LLM benchmark uses binary rubric criteria specifically to reduce scorer variance. Holistic judgments ("is this good Rust?") have high variance between LLM calls. Binary questions ("does it use openraft?") produce consistent scores across runs.
- Running N=5 per condition at temperature=0 is a deliberate variance-reduction choice. Temperature=0 is not fully deterministic on all models, but variance at temperature=0 is small enough that mean-of-5 produces a stable score.
- The `delta_product_vs_naive ≥ 0.15` threshold is the most important pass criterion. A product score of 0.90 is meaningless if naive also scores 0.87 — Product would be providing no incremental value. The delta threshold enforces that Product earns its place in the workflow.

**Rejected alternatives:**
- **Only property tests** — cannot test CLI surface, error formatting, exit codes, or concurrent behaviour.
- **Only integration tests** — hand-crafted inputs miss parser edge cases and graph algorithm boundary conditions that proptest finds routinely.
- **Only the LLM benchmark** — high cost, slow feedback loop. Unsuitable as a development-time safety net. The property and integration tests must run on every commit.
- **Manual LLM evaluation** — subjective, unrepeatable, non-comparable across releases. The rubric-based approach is mechanical and produces a number that can be tracked over time.


---