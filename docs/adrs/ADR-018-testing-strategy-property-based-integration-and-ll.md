---
id: ADR-018
title: Testing Strategy — Property-Based, Session-Based, and LLM Benchmark
status: accepted
features: []
supersedes: []
superseded-by: []
domains:
- testing
scope: cross-cutting
content-hash: sha256:cbd88ee3c19d2d0ac563cffb7514cc3ef0ce573a63904fe13e281998b4bed022
amendments:
- date: 2026-04-17T19:26:17Z
  reason: Replace Design 2 (Rust fixture-based Integration Test Harness) with Design 2 (Session-Based Integration Testing built on the request model from ADR-038). The original harness wrote raw YAML strings to disk — a second copy of the front-matter schema that had to be maintained in parallel with the product. With the request interface (ADR-038) now shipping, session tests can build fixtures through the same apply pipeline real users and agents use. Adds TC-P012..TC-P014 for request-model property invariants (failed apply leaves zero files, append is idempotent, forward-ref resolution is deterministic). Adds the canonical session library (ST-001..ST-083) and updates phase assignments. Also retitles the ADR to Property-Based, Session-Based, and LLM Benchmark to match the new Design 2. Captured as an amendment rather than a supersession because the three-design structure, the property/benchmark designs, and the rationale all carry forward unchanged.
  previous-hash: sha256:660cd17f84e0773b44d486846c5fc60addb23fb787907bfad77fcf5a2d9b2c3a
- date: 2026-04-22T10:45:37Z
  reason: Promote scope from domain to cross-cutting and add the testing domain. Session-based testing, property tests, and LLM benchmarking are expectations on every feature, not decisions inside a single slice. Flipping to cross-cutting activates the ADR-025 forcing function so new feature requests must link or acknowledge this ADR. The three-design structure, the property/session/benchmark designs, and the rationale all carry forward unchanged, so this is an amendment rather than a supersession.
  previous-hash: sha256:08c82f15ce4c6f556bd79c20b24df77631837ea7c0bce8c9a04d76862e365b39
- date: 2026-04-22T12:50:49Z
  reason: 'Drop stale ST-055 and ST-056 session entries. Those entries predated ADR-032 (content-hash immutability) and ADR-034 (lifecycle gate), which reassigned W016 and W017 to different canonical meanings: W016 = accepted ADR has no content-hash, W017 = complete feature with proposed ADR. The original claims (body-change after complete emits W017; new TC after complete emits W016) describe checks that were never implemented. The two actual W-code scenarios are covered by integration tests TC-424 and TC-442. Shrinks the Session library range from ST-050..ST-056 to ST-050..ST-054 and keeps ST-053/054 since they map to real drift behaviour. Captured as an amendment because the three-design structure and rationale are unchanged.'
  previous-hash: sha256:41096132e46d9ac3ea4b29f56828d8afe050a1e2a536f73c8efda2e0a097ca0a
---

**Status:** Accepted (amended to replace Design 2 with session-based integration testing and add request-model property invariants)

**Context:** Product has three distinct failure classes that require three distinct testing approaches:

1. **Algorithmic correctness** — graph algorithms (topological sort, betweenness centrality, BFS, reachability) and the front-matter parser must produce correct results for all valid inputs, not just the ones the test author thought to write. Unit tests on hand-crafted inputs cannot cover the boundary cases that distributed systems and parser edge cases produce.

2. **Command correctness** — the full CLI surface (argument parsing, file I/O, error formatting, exit codes, stdout/stderr separation) must behave correctly on real repository state. Algorithmic unit tests cannot catch bugs in how the CLI routes a subcommand, formats a diagnostic message, or handles a concurrent write.

3. **Value delivery** — the core claim of Product is that context bundles improve LLM implementation quality. This claim is currently unvalidated. If context bundles do not measurably improve agent outputs, the product's fundamental design assumption is wrong and must be revised.

The original Design 2 used a Rust harness that built fixture repositories by writing raw YAML strings. With the introduction of the request model (ADR-038), there is a better primitive: the request YAML itself. A session test builds repository state through the same interface real users and agents use — create and change requests — and then asserts on the resulting state. This is a stronger test: if `product request apply` is broken, the session fails immediately. If the underlying commands are broken, the session fails when it reaches them. The fixture-writing layer and the command-under-test are no longer distinct.

No single testing approach covers all three failure classes. This ADR specifies all three, defines their scope boundaries, and assigns them to phases. The full testing specification lives in [`docs/product-testing-spec.md`](../product-testing-spec.md); this ADR pins the decisions that shape that spec.

---

### Design 1: Property-Based Testing (proptest)

**Target failure class:** Algorithmic correctness — inputs the test author did not anticipate.

**Tool:** `proptest` crate. Generates thousands of random inputs satisfying user-defined strategies, shrinks failing inputs to minimal reproducible examples.

**Scope:** Pure functions only — graph construction, traversal algorithms, front-matter parser, file write logic, request resolution. No filesystem, no CLI, no network.

**Repository location:** `tests/property/` — separate from unit tests to allow independent execution and longer run budgets.

#### Property Set

**Parser robustness (from ADR-013):**

| TC | Property |
|---|---|
| TC-P001 | No input causes a panic |
| TC-P002 | Valid front-matter round-trips |
| TC-P003 | Unknown fields preserved on write |
| TC-P004 | Malformed input returns structured error |

**Graph algorithm correctness (from ADR-012):**

| TC | Property |
|---|---|
| TC-P005 | Topo order respects all dependency edges |
| TC-P006 | Topo sort detects all cycles |
| TC-P007 | Centrality always in range |
| TC-P008 | Reverse reachability inverts forward |
| TC-P009 | BFS deduplication — node appears once |

**File write safety (from ADR-015):**

| TC | Property |
|---|---|
| TC-P010 | Atomic write — no torn state |
| TC-P011 | Write + re-read is identity |

**Request model invariants (from ADR-038, added by amendment):**

| TC | Property |
|---|---|
| TC-P012 | Failed apply leaves zero files changed |
| TC-P013 | Append is idempotent |
| TC-P014 | Forward-ref resolution is deterministic |

Configuration:

```toml
[proptest]
cases = 1000
max_shrink_iters = 500
failure_persistence = "file"
```

---

### Design 2: Session-Based Integration Testing

**Target failure class:** Command correctness — full CLI behaviour on real repository state.

**The key principle:** Session tests build repository state through the request model, then assert on graph state, file content, and command output. The same interface real users and agents use is the test fixture mechanism. There is no separate fixture-writing layer.

**Scope:** Full binary execution. Every session runs the compiled `product` binary against a real temporary directory. No mocking. No hand-written YAML strings.

**Repository location:** `tests/sessions/`

#### Session Runner

```rust
pub struct Session { /* temp dir, bin path, step counter */ }

impl Session {
    pub fn new() -> Self;
    pub fn apply(&mut self, request_yaml: &str) -> ApplyResult;
    pub fn apply_file(&mut self, path: &str) -> ApplyResult;
    pub fn run(&self, args: &[&str]) -> Output;
    pub fn assert_file_exists(&self, path: &str) -> &Self;
    pub fn assert_frontmatter(&self, path: &str, field: &str, value: &str) -> &Self;
    pub fn assert_array_contains(&self, path: &str, field: &str, value: &str) -> &Self;
    pub fn assert_graph_clean(&self) -> &Self;
    pub fn assert_graph_error(&self, code: &str) -> &Self;
    pub fn assert_graph_warning(&self, code: &str) -> &Self;
    pub fn assert_tag_exists(&self, tag: &str) -> &Self;
    pub fn assert_no_tag(&self, tag: &str) -> &Self;
    pub fn sparql(&self, query: &str) -> Vec<HashMap<String, String>>;
}

pub struct ApplyResult {
    pub applied:  bool,
    pub created:  Vec<AssignedArtifact>,
    pub changed:  Vec<ChangedArtifact>,
    pub findings: Vec<Finding>,
}

impl ApplyResult {
    pub fn assert_applied(&self) -> &Self;
    pub fn assert_failed(&self) -> &Self;
    pub fn assert_finding(&self, code: &str) -> &Self;
    pub fn assert_no_finding(&self, code: &str) -> &Self;
    pub fn id_for(&self, ref_name: &str) -> String;
    pub fn assert_clean(&self) -> &Self;
}
```

The `ApplyResult.id_for(ref)` method is the key ergonomic improvement over the fixture-based harness: tests no longer hardcode artifact IDs — they ask the request interface for the ID assigned to a declared `ref:` name and use it throughout.

#### Canonical Session Library

Sessions are the primary way to describe expected Product behaviour. Every significant workflow has a session. Groups and full names are documented in `docs/product-testing-spec.md`:

- **ST-001..ST-006** — create operations (forward refs, cross-links, bidirectionality, ID assignment order)
- **ST-010..ST-015** — change operations (append/remove/set/delete on scalars, arrays, dot-notation)
- **ST-020..ST-022** — atomicity (zero files on failure, mid-write recovery, lock serialisation)
- **ST-030..ST-035** — validation (E002, E003, E011, E012, E013, vocabulary)
- **ST-040..ST-042** — phase gate behaviour
- **ST-050..ST-054** — verification, drift, completion tags
- **ST-060..ST-062** — context bundles
- **ST-070..ST-072** — domain coverage
- **ST-080..ST-083** — full end-to-end workflows

Session files are dual-purpose: they are tests, and they are documentation. The request YAMLs in `tests/sessions/ST-001/` are the same format shown in the quickstart guide.

---

### Design 3: LLM Context Quality Benchmark

**Target failure class:** Value delivery — does Product actually improve LLM implementation quality?

**Scope:** End-to-end quality measurement. Runs the compiled binary to generate context bundles, sends them to an LLM, scores the output against a rubric using a separate LLM call.

**Repository location:** `benchmarks/`

**Run cadence:** Not in CI. Triggered manually on release candidates, after context bundle format changes (ADR-006, ADR-011, ADR-012), and monthly for trend tracking.

#### Three Conditions

| Condition | Context provided |
|---|---|
| `none` | No context beyond the prompt |
| `naive` | Full prd + adrs documents concatenated |
| `product` | Output of `product context FT-XXX --depth 2` |

#### Scoring Protocol

Each rubric criterion is scored by a separate LLM call with a narrow binary question. Final score = Σ(satisfied_criteria × weight) / Σ(all_criteria × weight). Each condition run N=5 times at temperature=0, reported as the mean.

#### Pass Thresholds

- `score(product) ≥ 0.80` — absolute quality threshold
- `score(product) - score(naive) ≥ 0.15` — Product must add measurable value

Both must hold.

#### Initial Task Set (Phase 3)

| TC | Task | Feature | Key rubric dimension |
|---|---|---|---|
| TC-030 | Raft leader election | FT-001 | Architecture compliance |
| TC-031 | Front-matter parser | FT-Product-001 | Robustness |
| TC-032 | Context bundle assembly | FT-Product-002 | Correctness |

---

### Testing Phase Assignment

| Design | Phase | Prerequisite |
|---|---|---|
| Session runner infrastructure | Phase 1 | Binary compiles, request apply works |
| Sessions: create operations (ST-001–ST-006) | Phase 1 | `product request apply` implemented |
| Sessions: atomicity (ST-020–ST-022) | Phase 1 | Atomic writes implemented |
| Sessions: validation (ST-030–ST-035) | Phase 1 | Validation rules implemented |
| Property: parser robustness (TC-P001–TC-P004) | Phase 1 | Parser implemented |
| Property: file safety (TC-P010–TC-P011) | Phase 1 | Atomic writes implemented |
| Property: request invariants (TC-P012–TC-P014) | Phase 1 | Request model implemented |
| Sessions: change operations (ST-010–ST-015) | Phase 2 | `product request change` implemented |
| Sessions: phase gate (ST-040–ST-042) | Phase 2 | Phase gate implemented |
| Sessions: context bundles (ST-060–ST-062) | Phase 2 | Context assembly implemented |
| Property: graph algorithms (TC-P005–TC-P009) | Phase 2 | Algorithms implemented |
| Sessions: verification and drift (ST-050–ST-054) | Phase 3 | `product verify`, git tags |
| Sessions: domain coverage (ST-070–ST-072) | Phase 3 | Preflight implemented |
| Sessions: full workflows (ST-080–ST-083) | Phase 3 | All commands implemented |
| LLM benchmark (TC-030–TC-032) | Phase 3 | Context bundles complete |

---

**Rationale:**

- **Three separate designs** remain necessary because each catches a disjoint failure class. Collapsing them into one approach would leave two failure classes untested.
- **Session-based testing replaces the fixture-based harness** because sessions use the same interface real users and agents use. A fixture that writes raw YAML strings to disk is testing the parser and file layout, not the product. A session that applies a create request and then asserts on the result is testing the full stack — request validation, ID assignment, atomic write, graph construction — in one coherent flow.
- **Session files double as documentation.** The request YAMLs in `tests/sessions/ST-001/` are the same format shown in the quickstart guide. A reader learning Product reads the session and immediately understands the complete interaction model.
- **The `ApplyResult.id_for(ref)` method** is the key ergonomic improvement. Tests never hardcode IDs; they ask the request layer for the ID assigned to a declared `ref:` name. This is the same forward-reference model the request YAML itself uses — consistent from authoring to testing.
- **Property tests remain on pure functions.** Attempting to property-test the full CLI through sessions would be slow and produce unhelpful failures. The division is clean: sessions cover the full request→apply→assert loop; property tests verify the correctness of individual algorithms.
- **The LLM benchmark is unchanged.** It tests a different failure class (value delivery) and has no dependency on the request model.

**Rejected alternatives:**

- **Keep the fixture-based harness.** The old harness wrote raw YAML strings, duplicating the front-matter schema in a second location. When the schema changed, both the harness and the spec had to update. The session model derives its fixtures from the same schema the product uses. Rejected.
- **Only property tests.** Cannot test CLI surface, error formatting, exit codes, or concurrent behaviour.
- **Only session tests.** Hand-crafted inputs miss parser edge cases and graph-algorithm boundary conditions that proptest finds routinely.
- **Only the LLM benchmark.** High cost, slow feedback loop. Unsuitable as a development-time safety net. The property and session tests must run on every commit.
- **Manual LLM evaluation.** Subjective, unrepeatable, non-comparable across releases. The rubric-based approach is mechanical and produces a number that can be tracked over time.
- **Golden file tests for sessions.** Session assertions are explicit conditions, not file snapshots. Golden files accumulate churn when IDs change. Explicit assertions (`assert_array_contains`, `assert_frontmatter`) are more readable and more stable. Rejected.
- **Session files as separate Rust files per step.** Embedding the request YAML inline in Rust source keeps tests readable. A separate file per session step adds filesystem overhead without benefit. Rejected.

---

### Scope — Cross-Cutting

This ADR is classified **cross-cutting** under ADR-025. The three designs are expectations on every feature, not decisions inside one slice:

- Any feature that adds pure algorithmic logic should declare its property tests (Design 1) or acknowledge the absence in `domains-acknowledged`.
- Any feature that adds CLI, request-model, or graph behaviour should land with a session in `tests/sessions/` (Design 2) or acknowledge the gap.
- Context-bundle-shaping features should include or update a benchmark task (Design 3) when they change bundle content.

Cross-cutting scope activates the ADR-025 forcing function: new feature requests are surfaced for test-strategy review and must link or acknowledge this ADR. The three existing links (FT-015, FT-025, FT-043) remain; the change is prospective — future features pick up the obligation, completed features are not retroactively required to amend.

---
