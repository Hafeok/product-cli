# Product Testing Specification

> Standalone reference for the Product testing strategy.
> Extracted from ADR-018 in `product-adrs.md`.
> Covers: session-based integration testing, property-based testing, and LLM benchmark.

---

## ADR-018: Testing Strategy — Property-Based, Session-Based, and LLM Benchmark

**Status:** Accepted (amended to add session-based integration testing)

**Context:** Product has three distinct failure classes that require three distinct testing approaches:

1. **Algorithmic correctness** — graph algorithms (topological sort, betweenness centrality, BFS, reachability) and the front-matter parser must produce correct results for all valid inputs, not just the ones the test author thought to write. Unit tests on hand-crafted inputs cannot cover the boundary cases that distributed systems and parser edge cases produce.

2. **Command correctness** — the full CLI surface (argument parsing, file I/O, error formatting, exit codes, stdout/stderr separation) must behave correctly on real repository state. Algorithmic unit tests cannot catch bugs in how the CLI routes a subcommand, formats a diagnostic message, or handles a concurrent write.

3. **Value delivery** — the core claim of Product is that context bundles improve LLM implementation quality. This claim is currently unvalidated. If context bundles do not measurably improve agent outputs, the product's fundamental design assumption is wrong and must be revised.

The original Design 2 used a Rust harness that built fixture repositories by writing raw YAML strings. With the introduction of the request model (ADR-032), there is a better primitive: the request YAML itself. A session test builds repository state through the same interface real users and agents use — create and change requests — and then asserts on the resulting state. This is a stronger test: if `product request apply` is broken, the session fails immediately. If the underlying commands are broken, the session fails when it reaches them. The fixture-writing layer and the command-under-test are no longer distinct.

No single testing approach covers all three failure classes. This ADR specifies all three, defines their scope boundaries, and assigns them to phases.

---

### Design 1: Property-Based Testing (proptest)

**Target failure class:** Algorithmic correctness — inputs the test author did not anticipate.

**Tool:** `proptest` crate. Generates thousands of random inputs satisfying user-defined strategies, shrinks failing inputs to minimal reproducible examples.

**Scope:** Pure functions only — graph construction, traversal algorithms, front-matter parser, file write logic. No filesystem, no CLI, no network.

**Repository location:** `tests/property/`

#### Generators

```rust
/// Generates a valid DAG of Feature nodes.
fn arb_dag(size: impl Strategy<Value = usize>, edge_density: f64)
  -> impl Strategy<Value = FeatureGraph>

/// Generates a connected graph — required for centrality to be meaningful.
fn arb_connected_graph(size: impl Strategy<Value = usize>, density: f64)
  -> impl Strategy<Value = FeatureGraph>

/// Generates syntactically valid Feature structs.
fn arb_valid_feature() -> impl Strategy<Value = Feature>

/// Generates arbitrary byte strings including edge cases.
fn arb_arbitrary_input() -> impl Strategy<Value = String>

/// Generates a valid YAML key-value pair not in the Product schema.
fn arb_unknown_field() -> impl Strategy<Value = (String, String)>
```

#### Property Set

**Parser robustness:**

| TC | Property | Formal expression |
|---|---|---|
| TC-P001 | No input causes a panic | `∀s:String: parse_frontmatter(s) ≠ panic` |
| TC-P002 | Valid front-matter round-trips | `∀f:Feature: parse(serialise(f)) = f` |
| TC-P003 | Unknown fields preserved on write | `∀f:Feature, k:UnknownField: serialise(inject(f,k)) ⊇ k` |
| TC-P004 | Malformed input returns structured error | `∀s:InvalidYAML: parse(s) = Err(E001)` |

**Graph algorithm correctness:**

| TC | Property | Formal expression |
|---|---|---|
| TC-P005 | Topo order respects all dependency edges | `∀g:DAG, (u,v)∈g.edges: pos(topo(g),u) < pos(topo(g),v)` |
| TC-P006 | Topo sort detects all cycles | `∀g:CyclicGraph: topo_sort(g) = Err(E003)` |
| TC-P007 | Centrality always in range | `∀g:ConnectedGraph, n∈g.nodes: 0.0 ≤ centrality(g,n) ≤ 1.0` |
| TC-P008 | Reverse reachability inverts forward | `∀g:Graph, u,v∈g.nodes: reachable(g,u,v) ↔ reachable(rev(g),v,u)` |
| TC-P009 | BFS deduplication — node appears once | `∀g:Graph, seed:Node, d:Depth: |{n \| n∈bfs(g,seed,d)}| = |bfs(g,seed,d)|` |

**File write safety:**

| TC | Property | Formal expression |
|---|---|---|
| TC-P010 | Atomic write — no torn state | `∀content:String, cutAt:Offset: file_after_interrupt(cutAt) ∈ {original, new}` |
| TC-P011 | Write + re-read is identity | `∀content:String: read(atomic_write(path, content)) = content` |

**Request model invariants:**

| TC | Property | Formal expression |
|---|---|---|
| TC-P012 | Failed apply leaves zero files changed | `∀r:Request, ¬valid(r): files_after_apply(r) = files_before_apply(r)` |
| TC-P013 | Append is idempotent | `∀r:AppendRequest: apply(apply(r)) = apply(r)` |
| TC-P014 | Forward ref resolution is deterministic | `∀r:Request: resolve_refs(r) = resolve_refs(r)` |

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

#### Session Structure

A session is a directory containing ordered request files plus an assertion script:

```
tests/sessions/
  ST-001-create-feature-with-adr/
    README.md              # what this session tests and why
    01-create.yaml         # product request create
    02-change.yaml         # product request change
    03-assert.sh           # assertions: graph check, file content, command output
  ST-002-dep-requires-adr/
    README.md
    01-create-dep-no-adr.yaml   # intentionally invalid
    02-assert.sh                # assert validation failure, zero files written
  ST-003-phase-gate/
    README.md
    01-create-phase1.yaml
    02-verify-phase1.sh         # run product verify, mark TCs passing
    03-assert-gate-open.sh      # assert phase 2 now accessible
    04-create-phase2.yaml
    05-assert-phase2-linked.sh
```

Each step is a request YAML (applied via `product request apply`) or a shell script for
verification commands (`product verify`, `product graph check`, `product drift check`).

#### Session Runner

```rust
pub struct Session {
    dir: TempDir,
    bin: PathBuf,
    step: usize,
}

impl Session {
    pub fn new() -> Self
    pub fn apply(&mut self, request_yaml: &str) -> ApplyResult
    pub fn apply_file(&mut self, path: &str) -> ApplyResult
    pub fn run(&self, args: &[&str]) -> Output
    pub fn assert_file_exists(&self, path: &str) -> &Self
    pub fn assert_frontmatter(&self, path: &str, field: &str, value: &str) -> &Self
    pub fn assert_array_contains(&self, path: &str, field: &str, value: &str) -> &Self
    pub fn assert_graph_clean(&self) -> &Self       // product graph check exits 0 or 2
    pub fn assert_graph_error(&self, code: &str) -> &Self   // specific E-code present
    pub fn assert_graph_warning(&self, code: &str) -> &Self // specific W-code present
    pub fn assert_tag_exists(&self, tag: &str) -> &Self
    pub fn assert_no_tag(&self, tag: &str) -> &Self
    pub fn sparql(&self, query: &str) -> Vec<HashMap<String, String>> // query the graph
}

pub struct ApplyResult {
    pub applied:   bool,
    pub created:   Vec<AssignedArtifact>,
    pub changed:   Vec<ChangedArtifact>,
    pub findings:  Vec<Finding>,
}

impl ApplyResult {
    pub fn assert_applied(&self) -> &Self
    pub fn assert_failed(&self) -> &Self
    pub fn assert_finding(&self, code: &str) -> &Self
    pub fn assert_no_finding(&self, code: &str) -> &Self
    pub fn id_for(&self, ref_name: &str) -> String  // resolve ref → assigned ID
    pub fn assert_clean(&self) -> &Self             // applied: true, no findings
}
```

#### Writing a Session Test

```rust
// tests/sessions/ST-001-create-feature-with-adr.rs

#[test]
fn create_feature_with_adr_and_tc() {
    let mut session = Session::new();

    // Step 1: create feature + ADR + TC in one request
    let result = session.apply(r#"
        type: create
        reason: "Add cluster foundation"
        artifacts:
          - type: feature
            ref: ft-cluster
            title: Cluster Foundation
            phase: 1
            domains: [consensus]
            adrs: [ref:adr-raft]
            tests: [ref:tc-leader]

          - type: adr
            ref: adr-raft
            title: Use openraft for consensus
            domains: [consensus]
            scope: domain
            features: [ref:ft-cluster]

          - type: tc
            ref: tc-leader
            title: Leader elected within 10s
            tc-type: scenario
            validates:
              features: [ref:ft-cluster]
              adrs: [ref:adr-raft]
    "#);

    result.assert_applied();
    result.assert_clean();

    // Resolve assigned IDs
    let ft_id  = result.id_for("ft-cluster");
    let adr_id = result.id_for("adr-raft");
    let tc_id  = result.id_for("tc-leader");

    // Assert files exist
    session.assert_file_exists(&format!("docs/features/{}-cluster-foundation.md", ft_id));
    session.assert_file_exists(&format!("docs/adrs/{}-use-openraft.md", adr_id));

    // Assert cross-links are bidirectional
    session.assert_array_contains(
        &format!("docs/features/{}-cluster-foundation.md", ft_id),
        "adrs", &adr_id
    );
    session.assert_array_contains(
        &format!("docs/adrs/{}-use-openraft.md", adr_id),
        "features", &ft_id
    );

    // Assert graph is healthy
    session.assert_graph_clean();

    // Step 2: add a domain acknowledgement via change request
    let result2 = session.apply(&format!(r#"
        type: change
        reason: "Acknowledge networking domain"
        changes:
          - target: {}
            mutations:
              - op: append
                field: domains
                value: networking
              - op: set
                field: domains-acknowledged.networking
                value: "No network layer in phase 1"
    "#, ft_id));

    result2.assert_applied();
    session.assert_frontmatter(
        &format!("docs/features/{}-cluster-foundation.md", ft_id),
        "domains-acknowledged.networking",
        "No network layer in phase 1"
    );
}
```

#### Session Test Library — the canonical scenarios

Sessions are the primary way to describe expected Product behaviour. Every significant
workflow is a session. The session name is the specification:

```
tests/sessions/
  # Create operations
  ST-001  create-feature-with-adr-and-tc
  ST-002  create-dep-requires-governing-adr         # E013 on missing ADR
  ST-003  create-dep-with-adr-in-same-request       # E013 satisfied within request
  ST-004  create-with-forward-references            # ref: resolution
  ST-005  create-multiple-adrs-same-phase           # ID assignment order
  ST-006  create-cross-links-bidirectional          # features↔adrs↔tcs links

  # Change operations
  ST-010  change-append-domain                      # domains array mutation
  ST-011  change-set-acknowledgement                # nested field set
  ST-012  change-invalid-target                     # E002 on non-existent target
  ST-013  change-body-mutation                      # body field set
  ST-014  change-remove-from-array                  # op: remove
  ST-015  change-append-deduplicates                # idempotent append

  # Atomicity
  ST-020  failed-apply-leaves-zero-files            # validation error → no writes
  ST-021  failed-apply-mid-write-recovery           # simulated write failure
  ST-022  concurrent-apply-serialised               # advisory lock enforced

  # Validation
  ST-030  validation-e013-dep-no-adr
  ST-031  validation-e002-broken-ref
  ST-032  validation-e003-dep-cycle
  ST-033  validation-e012-unknown-domain
  ST-034  validation-e011-empty-acknowledgement
  ST-035  validation-domain-not-in-vocabulary

  # Phase gate
  ST-040  phase-gate-blocks-on-failing-exit-criteria
  ST-041  phase-gate-opens-after-verify
  ST-042  phase-gate-no-exit-criteria-always-open

  # Feature completion and drift
  ST-050  verify-creates-completion-tag
  ST-051  verify-complete-feature-status
  ST-052  verify-failing-tc-stays-in-progress
  ST-053  drift-check-detects-changes-since-tag
  ST-054  drift-check-no-tag-emits-w020
  ST-055  body-change-after-complete-emits-w017
  ST-056  new-tc-after-complete-emits-w016

  # Context bundles
  ST-060  context-includes-dependency-section
  ST-061  context-depth-2-includes-shared-adrs
  ST-062  context-measure-writes-bundle-block

  # Domain coverage
  ST-070  preflight-flags-missing-domain-adr
  ST-071  preflight-clean-after-acknowledge
  ST-072  graph-coverage-matrix-symbols

  # Full workflows (multi-session composition)
  ST-080  workflow-feature-from-idea-to-complete    # all 8 steps
  ST-081  workflow-add-dependency-to-existing-feature
  ST-082  workflow-supersede-adr
  ST-083  workflow-migration-then-request-links
```

#### Session files as documentation

Session files in `tests/sessions/` are the canonical description of how Product behaves.
When behaviour changes, the session file changes with it. The session name and README are
the spec; the YAML steps are the executable proof.

Session request files can be reused as examples in documentation: `ST-001/01-create.yaml`
is the same file that appears in the quickstart guide. The test and the example are the
same artifact.

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
    src/main.rs
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
      results.md
    latest -> 2026-04-11/
```

#### Task Structure

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

| Condition | Context provided |
|---|---|
| `none` | No context beyond the prompt |
| `naive` | Full `picloud-prd.md` + `picloud-adrs.md` concatenated |
| `product` | Output of `product context FT-001 --depth 2` |

#### Scoring Protocol

Each rubric criterion is scored by a separate LLM call with a narrow binary question:

```
"Does the following implementation satisfy this criterion:
'Uses openraft crate, not a custom Raft implementation'?
Answer only YES or NO."
```

Final score = Σ(satisfied_criteria × weight) / Σ(all_criteria × weight)

Each condition is run N=5 times at temperature=0. Reported score is the mean.

#### Pass Thresholds

- `score(product) ≥ 0.80` — absolute quality threshold
- `score(product) - score(naive) ≥ 0.15` — Product must add measurable value

Both conditions must hold.

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
      "conditions": {
        "none":    { "mean": 0.41 },
        "naive":   { "mean": 0.63 },
        "product": { "mean": 0.87 }
      },
      "delta_product_vs_naive": 0.24,
      "pass": true
    }
  ]
}
```

#### Initial Task Set (Phase 3)

| TC | Task | Feature | Key rubric dimension |
|---|---|---|---|
| TC-030 | Raft leader election | FT-001 | Architecture compliance (openraft, RDF) |
| TC-031 | Front-matter parser | FT-Product-001 | Robustness (no panics, error codes) |
| TC-032 | Context bundle assembly | FT-Product-002 | Correctness (depth, dedup, ordering) |

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
| Sessions: verification and drift (ST-050–ST-056) | Phase 3 | `product verify`, git tags |
| Sessions: domain coverage (ST-070–ST-072) | Phase 3 | Preflight implemented |
| Sessions: full workflows (ST-080–ST-083) | Phase 3 | All commands implemented |
| LLM benchmark (TC-030–TC-032) | Phase 3 | Context bundles complete |

---

**Rationale:**
- Session-based testing replaces fixture-based testing as the primary integration approach because sessions use the same interface real users and agents use. A fixture that writes raw YAML strings to disk is testing the parser and file layout, not the product. A session that applies a create request and then asserts on the result is testing the full stack — request validation, ID assignment, atomic write, graph construction — in one coherent flow.
- Session files are documentation as well as tests. The request YAMLs in `tests/sessions/ST-001/` are the same format shown in the quickstart guide. A reader learning Product reads the session and immediately understands the complete interaction model, not a Rust fixture API they need to translate.
- The `ApplyResult.id_for(ref)` method is the key ergonomic improvement. In the old harness, tests had to know or hardcode artifact IDs. In the session model, the test asks for the assigned ID by reference name and uses it throughout. This is the same forward-reference model the request YAML uses — consistent from authoring to testing.
- Property tests remain on pure functions. Attempting to property-test the full CLI through sessions would be slow and produce unhelpful failures. The division is clean: sessions test the complete request→apply→assert loop; property tests verify the correctness of individual algorithms.
- The LLM benchmark is unchanged. It tests a different failure class (value delivery) and has no dependency on the request model.

**Rejected alternatives:**
- **Keep fixture-based harness** — the old harness writes raw YAML strings, which duplicates the front-matter schema in a second location. When the schema changes, both the harness and the spec must update. The session model derives its fixtures from the same schema the product uses. Rejected.
- **Golden file tests for sessions** — session assertions are explicit conditions, not file snapshots. Golden files accumulate churn when IDs change. Explicit assertions (`assert_array_contains`, `assert_frontmatter`) are more readable and more stable. Rejected.
- **Session files as Rust** — embedding the request YAML inline in Rust source (as shown above) keeps tests readable. A separate file per session step adds filesystem overhead without benefit. Rejected.



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

---

## Session Index

Quick reference of all planned sessions from the library above:

ST-001  create-feature-with-adr-and-tc
ST-002  create-dep-requires-governing-adr         # E013 on missing ADR
ST-003  create-dep-with-adr-in-same-request       # E013 satisfied within request
ST-004  create-with-forward-references            # ref: resolution
ST-005  create-multiple-adrs-same-phase           # ID assignment order
ST-006  create-cross-links-bidirectional          # features↔adrs↔tcs links
ST-010  change-append-domain                      # domains array mutation
ST-011  change-set-acknowledgement                # nested field set
ST-012  change-invalid-target                     # E002 on non-existent target
ST-013  change-body-mutation                      # body field set
ST-014  change-remove-from-array                  # op: remove
ST-015  change-append-deduplicates                # idempotent append
ST-020  failed-apply-leaves-zero-files            # validation error → no writes
ST-021  failed-apply-mid-write-recovery           # simulated write failure
ST-022  concurrent-apply-serialised               # advisory lock enforced
ST-030  validation-e013-dep-no-adr
ST-031  validation-e002-broken-ref
ST-032  validation-e003-dep-cycle
ST-033  validation-e012-unknown-domain
ST-034  validation-e011-empty-acknowledgement
ST-035  validation-domain-not-in-vocabulary
ST-040  phase-gate-blocks-on-failing-exit-criteria
ST-041  phase-gate-opens-after-verify
ST-042  phase-gate-no-exit-criteria-always-open
ST-050  verify-creates-completion-tag
ST-051  verify-complete-feature-status
ST-052  verify-failing-tc-stays-in-progress
ST-053  drift-check-detects-changes-since-tag
ST-054  drift-check-no-tag-emits-w020
ST-055  body-change-after-complete-emits-w017
ST-056  new-tc-after-complete-emits-w016
ST-060  context-includes-dependency-section
ST-061  context-depth-2-includes-shared-adrs
ST-062  context-measure-writes-bundle-block
ST-070  preflight-flags-missing-domain-adr
ST-071  preflight-clean-after-acknowledge
ST-072  graph-coverage-matrix-symbols
ST-080  workflow-feature-from-idea-to-complete    # all 8 steps
ST-081  workflow-add-dependency-to-existing-feature
ST-082  workflow-supersede-adr
ST-083  workflow-migration-then-request-links
