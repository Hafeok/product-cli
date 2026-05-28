---
id: FT-043
title: Session-Based Integration Testing
phase: 5
status: complete
depends-on:
- FT-015
- FT-018
- FT-041
- FT-042
adrs:
- ADR-009
- ADR-013
- ADR-015
- ADR-018
- ADR-038
- ADR-039
tests:
- TC-530
- TC-531
- TC-532
- TC-533
- TC-534
- TC-535
- TC-536
- TC-537
- TC-538
- TC-539
- TC-540
- TC-541
- TC-542
- TC-543
- TC-544
- TC-545
- TC-546
- TC-547
- TC-548
- TC-549
- TC-550
- TC-551
- TC-665
- TC-666
- TC-667
- TC-668
- TC-669
- TC-670
- TC-671
- TC-672
- TC-673
- TC-674
- TC-675
- TC-676
- TC-677
- TC-678
- TC-679
- TC-680
domains:
- api
- error-handling
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

Session-based integration testing is the primary way Product validates end-to-end command correctness. A **session** is a short Rust test function that drives a temporary repository through one or more `product request apply` calls and asserts on the resulting graph state, file content, and command output. The session builds its fixtures through the same interface real users and agents use — there is no separate fixture-writing layer.

The full design is captured in [`docs/product-testing-spec.md`](/docs/product-testing-spec.md) (extracted from ADR-018 as amended). This feature delivers the harness infrastructure, the canonical session library, and the property-test coverage for request-model invariants.

---

## Depends on

- **FT-041** — Product Request — Unified Write Interface. Sessions are built on `product request apply`; without the request interface, sessions cannot compose realistic fixtures atomically.
- **FT-042** — Request Log Hash-Chain and Replay. Some sessions (verify / drift / log-integrity families) assert on `requests.jsonl` entries after apply.
- **FT-018** — Validation and graph health. E-code assertions (E002, E003, E011, E012, E013) are the vocabulary session validation tests use.
- **FT-015** — Test Criteria. Each session and each property is a TC in its own right.

---

## Scope of this feature

### In

1. **The `Session` harness.** A `Session` struct that manages a `TempDir`, resolves the compiled `product` binary, and exposes a fluent assertion API: `assert_file_exists`, `assert_frontmatter`, `assert_array_contains`, `assert_graph_clean`, `assert_graph_error(code)`, `assert_graph_warning(code)`, `assert_tag_exists`, `assert_no_tag`, `sparql(query)`. Method signatures match the spec at `docs/product-testing-spec.md` § Session Runner.
2. **The `ApplyResult` return type.** `Session::apply(request_yaml)` returns an `ApplyResult` with `applied`, `created`, `changed`, `findings`. Methods: `assert_applied`, `assert_failed`, `assert_finding(code)`, `assert_no_finding(code)`, `id_for(ref)`, `assert_clean`. The `id_for(ref)` method resolves a declared `ref:` name to the ID assigned on apply — tests never hardcode IDs.
3. **Session directory layout.** Sessions live in `tests/sessions/` one file per session, named `tests/sessions/st_xxx_slug.rs` (or grouped by category). Each session test function carries the `#[test]` attribute and is runnable via `cargo test --test sessions`.
4. **The core Phase 1 session library — create, atomicity, validation families.** TC-533 through TC-547 implement ST-001..ST-006, ST-020..ST-022, and ST-030..ST-035 from the testing spec. Each session is explicit and self-contained; its README-equivalent prose lives in the session TC's body.
5. **Property-test coverage for request invariants.** TC-548/TC-549/TC-550 (TC-P012, TC-P013, TC-P014) live in `tests/property.rs` using `proptest`:
    - **TC-P012** — every request with any E-class finding produces zero changes on disk (verified by pre/post checksum of all artifact files).
    - **TC-P013** — applying the same `append` request twice produces the same end state.
    - **TC-P014** — resolving forward references in a request with the same declared artifacts produces the same assigned-ID ordering across runs.
6. **Harness self-tests.** TC-530/TC-531/TC-532 verify that the harness itself works: the Session API is present and typed as documented, `Session::run` actually invokes the product binary, and `ApplyResult::id_for` returns the real assigned ID for a declared ref.
7. **Runner configuration on every TC.** Every test written for this feature gets `runner: cargo-test` and `runner-args: "tc_XXX_snake_case"` in its front-matter at the same time the test is written, per CLAUDE.md's TC Runner Configuration policy.

### Out

- **Later session families.** ST-010..ST-015 (change operations), ST-040..ST-042 (phase gate), ST-050..ST-056 (verify / drift), ST-060..ST-062 (context bundles), ST-070..ST-072 (domain coverage), ST-080..ST-083 (full workflows) are scoped to a follow-on feature once their underlying Product features ship new session-observable surface area. The testing spec names them all; their test criteria will be added later.
- **LLM benchmark tasks (TC-030, TC-031, TC-032).** Already scoped under FT-025 "Benchmarks". No changes to that feature.
- **Graph-algorithm properties (TC-P005..TC-P009).** Already scoped under earlier phase-2 feature work.
- **Migration of the existing `tests/integration.rs`** into sessions. The existing integration tests continue to run unchanged; session tests are additive. A future cleanup feature may migrate or delete the older file once equivalent session coverage exists.
- **Golden-file tests for session output.** Session assertions are explicit conditions (`assert_array_contains`, `assert_frontmatter`). Golden files churn when IDs change; the spec rejects them for sessions.

---

## Harness API sketch

```rust
// tests/sessions/harness.rs

pub struct Session {
    dir: tempfile::TempDir,
    bin: std::path::PathBuf,
    step: usize,
}

impl Session {
    pub fn new() -> Self;
    pub fn apply(&mut self, request_yaml: &str) -> ApplyResult;
    pub fn apply_file(&mut self, path: &str) -> ApplyResult;
    pub fn run(&self, args: &[&str]) -> Output;
    pub fn assert_file_exists(&self, path: &str) -> &Self;
    pub fn assert_frontmatter(&self, path: &str, field: &str, value: &str) -> &Self;
    pub fn assert_array_contains(&self, path: &str, field: &str, value: &str) -> &Self;
    pub fn assert_graph_clean(&self) -> &Self;                      // exit 0 or 2
    pub fn assert_graph_error(&self, code: &str) -> &Self;          // specific E-code
    pub fn assert_graph_warning(&self, code: &str) -> &Self;        // specific W-code
    pub fn assert_tag_exists(&self, tag: &str) -> &Self;
    pub fn assert_no_tag(&self, tag: &str) -> &Self;
    pub fn sparql(&self, query: &str) -> Vec<std::collections::HashMap<String, String>>;
}

pub struct ApplyResult {
    pub applied:  bool,
    pub created:  Vec<AssignedArtifact>,   // ref_name, id, file
    pub changed:  Vec<ChangedArtifact>,
    pub findings: Vec<Finding>,            // code, severity, message, location
}

impl ApplyResult {
    pub fn assert_applied(&self) -> &Self;
    pub fn assert_failed(&self) -> &Self;
    pub fn assert_finding(&self, code: &str) -> &Self;
    pub fn assert_no_finding(&self, code: &str) -> &Self;
    pub fn id_for(&self, ref_name: &str) -> String;
    pub fn assert_clean(&self) -> &Self;   // applied && no E/W findings
}
```

The harness implementation reuses the existing `assert_cmd` + `tempfile` stack already present in `tests/integration.rs`. `apply()` invokes the compiled binary via `Session::run(["request", "apply", "<tmpfile>"])` and parses the JSON output produced with `--format json`.

---

## Acceptance criteria

A developer writing a new session test can:

1. Call `let mut s = Session::new()` and receive a fresh temp repository pre-initialised with `product init` semantics (a valid `product.toml`, empty `docs/features`, `docs/adrs`, `docs/tests`, `docs/deps`).
2. Call `let r = s.apply(r#"type: create ... "#)` with an inline YAML string and receive an `ApplyResult` whose `created` array lists every artifact written by the request.
3. Call `r.id_for("ft-cluster")` on a ref declared in the request YAML and receive the assigned ID (e.g. `"FT-044"`) — never a placeholder.
4. Use the returned ID in subsequent `s.assert_file_exists(...)` / `s.assert_frontmatter(...)` / `s.assert_array_contains(...)` / `s.run(&["context", &id])` calls.
5. Run `cargo test --test sessions` and observe every session test pass, with per-session clear failure messages when assertions break (including the offending path, field, expected, actual).
6. Chain assertions in a fluent style: `s.assert_file_exists(&a).assert_array_contains(&b, "adrs", &adr_id).assert_graph_clean();`.
7. Observe that the property tests TC-P012/TC-P013/TC-P014 generate thousands of inputs per run via `proptest` and never panic.

---

## Implementation notes

- **New test crate module: `tests/sessions/mod.rs`.** Houses `Session`, `ApplyResult`, `AssignedArtifact`, `ChangedArtifact`, `Finding`, `Output`. Each session test is a free function in its own file under `tests/sessions/`.
- **Invoke product with `--format json`** so `apply()` can parse the output and expose structured `created` / `changed` / `findings` arrays. The CLI already supports this flag (main.rs lines 15–20).
- **Bin resolution** uses `env!("CARGO_BIN_EXE_product")` — the same pattern `assert_cmd` uses internally. No environment variable dance.
- **`Session::new`** writes a minimal `product.toml` and creates the directory tree. Avoid calling the real `product init` command during harness bootstrap to keep session tests fast; if future sessions need `product init` behaviour they call it explicitly as a test step.
- **Property tests live in `tests/property.rs`**, the existing property-test file. TC-P012 requires a mini pre/post checksum helper that hashes every file under `docs/`; TC-P014 requires a ref-sorting assertion that runs `product request apply` twice against two identical repos and compares the assigned-ID arrays.
- **Runner config is mandatory on every TC for this feature.** After the test body is written, set `runner: cargo-test` and `runner-args: "tc_XXX_snake_case_title"` via `product_test_runner` (or via a follow-on `type: change` request). `product verify FT-043` will silently skip any TC lacking runner config.

---

## Follow-on work

Once the Phase 1 session library ships under this feature:

1. Backfill the Phase 2 session groups (change operations, phase gate, context bundles) as a new feature that depends on FT-043.
2. Backfill the Phase 3 session groups (verify / drift, domain coverage, full workflows) as another feature that depends on FT-043 plus the relevant Product features.
3. Update `scripts/generate-docs.sh` or the equivalent Diátaxis guide step to include a "How-to write a session test" section that points at `tests/sessions/` as the canonical example library.
4. Once Phase 2 and Phase 3 session coverage is in place, deprecate (or remove) the older `tests/integration.rs` path in a separate request.

---

## Description

Session-based integration testing is the primary mechanism for validating end-to-end `product` command correctness. A session is a Rust test function that drives a temporary repository through one or more `product request apply` calls and asserts on the resulting graph state, file content, and command output using a fluent `Session` / `ApplyResult` API. Sessions build fixtures through the same interface real users and agents use — there is no separate fixture layer. This feature delivers the harness infrastructure (`tests/sessions/`), the core Phase 1 session library (create, atomicity, and validation families), and property-test coverage for three request-model invariants (TC-P012, TC-P013, TC-P014).

## Functional Specification

### Inputs

- Rust test functions in `tests/sessions/` that call `Session::new()`, `Session::apply(request_yaml)`, and assertion methods.
- Inline YAML strings (or files via `Session::apply_file`) representing `product request` documents passed to the compiled `product` binary during each session step.
- The compiled `product` binary resolved via `env!("CARGO_BIN_EXE_product")` — no environment variable configuration required.
- `product.toml` and the standard directory tree written to a `TempDir` by `Session::new()` at harness bootstrap.
- For property tests: `proptest`-generated request YAML inputs exercising the three invariants (TC-P012: any E-class finding → zero disk changes; TC-P013: idempotent `append`; TC-P014: deterministic ref-to-ID ordering).

### Outputs

- **`cargo test --test sessions`** — runs every session test function, prints a per-session pass/fail result with clear failure messages naming the offending path, field, expected value, and actual value when an assertion breaks.
- **`ApplyResult`** — returned by `Session::apply`; carries `applied` (bool), `created` (vec of `AssignedArtifact` with `ref_name`, `id`, `file`), `changed` (vec of `ChangedArtifact`), and `findings` (vec of `Finding` with `code`, `severity`, `message`, `location`).
- **`Session` assertions** — fluent methods (`assert_file_exists`, `assert_frontmatter`, `assert_array_contains`, `assert_graph_clean`, `assert_graph_error`, `assert_graph_warning`, `assert_tag_exists`, `assert_no_tag`, `sparql`) return `&Self` for chaining; panic with a descriptive message on failure.
- **`ApplyResult::id_for(ref_name)`** — resolves a declared `ref:` name to the assigned ID (e.g. `"FT-044"`); the test never hardcodes IDs.

### State

- **`TempDir`** managed by each `Session` instance — contains a minimal `product.toml`, empty `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/deps/` directories, and `requests.jsonl` (created on first apply). Cleaned up when the `Session` is dropped.
- No persistent state outside the `TempDir`; sessions are fully isolated from one another and from the working tree.
- The compiled `product` binary on disk — the session harness invokes it as a subprocess; it is not modified by session tests.

### Behaviour

1. `Session::new()` creates a `TempDir`, writes a minimal `product.toml`, and creates the required directory tree. It does not call `product init` to keep bootstrap fast; tests that need `product init` behaviour call it explicitly as a step.
2. `Session::apply(request_yaml)` writes the YAML to a temp file, invokes `product request apply <tmpfile> --format json` via the compiled binary, parses the JSON response, and returns an `ApplyResult`. The `--format json` flag is required so `created` / `changed` / `findings` are structured.
3. `Session::run(args)` invokes the compiled binary with arbitrary arguments and returns the raw `Output`. Used for non-apply commands (`product graph check`, `product context FT-XXX`, etc.).
4. Assertion methods on `Session` and `ApplyResult` panic with descriptive messages on failure, including the offending path, field, expected, and actual values.
5. `ApplyResult::id_for(ref_name)` looks up the assigned ID in the `created` array; panics if the ref name is not present (catches accidental ref-name typos at test time).
6. Fluent chaining: `s.assert_file_exists(&a).assert_array_contains(&b, "adrs", &adr_id).assert_graph_clean()` — each method returns `&Self`.
7. Property tests in `tests/property.rs` use `proptest` to generate thousands of inputs per test run: TC-P012 pre/post checksums every file under `docs/`; TC-P013 applies the same request twice and diffs graph state; TC-P014 applies the same request to two identical repos and compares assigned-ID ordering.
8. Sessions in `tests/sessions/` are discovered as `#[test]` functions by `cargo test --test sessions`; each session is self-contained in its own file.

### Invariants

- A session test is fully isolated: it never reads from or writes to the working repository; all I/O goes through the `TempDir`.
- `ApplyResult::id_for(ref_name)` always returns a real assigned ID — never a placeholder or the ref name itself.
- `Session::new()` always produces a repository that `product request apply` can write to without preconditions.
- Every TC added for this feature has `runner: cargo-test` and `runner-args: "tc_XXX_snake_case"` in its front-matter at the same time the test is written (CLAUDE.md TC Runner Configuration policy).
- Property tests generate at least 1000 cases per run (proptest default); they must not panic under any generated input.

### Error handling

- Assertion failures in `Session` and `ApplyResult` methods panic immediately with a descriptive message; the panic propagates as a test failure.
- If `product request apply` returns a non-zero exit code when `assert_applied` is called, the method panics with the stdout and stderr from the subprocess.
- If `env!("CARGO_BIN_EXE_product")` resolves to a binary that does not exist (e.g. the binary was not compiled), the test panics at binary invocation with a clear "binary not found" message from `assert_cmd`.
- `ApplyResult::id_for` panics if the `ref_name` is not in the `created` array, surfacing ref-name typos immediately rather than propagating a wrong ID.
- No error codes are emitted by the harness itself; errors surface as test panics reported by the `cargo test` runner.

### Boundaries

- The `Session` harness invokes the compiled `product` binary as a subprocess; it does not call library functions directly. This is intentional: it validates the full binary surface including argument parsing and exit codes.
- `Session::new()` does not call `product init`; it writes a minimal fixture. Tests that need `product init` semantics (e.g. hook installation) call it explicitly.
- Session tests do not use golden files for output; assertions are explicit conditions. Golden files churn when IDs change and are rejected for sessions by the testing spec.
- Property tests live in `tests/property.rs`, not in `tests/sessions/`; they share the same compiled binary but use proptest rather than the `Session` harness.
- The harness does not mock the filesystem, the graph, or the `product` binary internals — it is a black-box integration layer.

## Out of scope

- Later session families (ST-010–ST-015 change operations, ST-040–ST-042 phase gate, ST-050–ST-056 verify/drift, ST-060–ST-062 context bundles, ST-070–ST-072 domain coverage, ST-080–ST-083 full workflows) are scoped to follow-on features once their underlying Product features ship observable surface area.
- LLM benchmark tasks (TC-030, TC-031, TC-032) — already scoped under FT-025 "Benchmarks".
- Graph-algorithm properties (TC-P005–TC-P009) — already scoped under earlier phase-2 feature work.
- Migration of the existing `tests/integration.rs` into sessions — the existing tests continue to run unchanged; session tests are additive. A future cleanup feature may migrate once equivalent session coverage exists.
- Golden-file tests for session output — rejected by the testing spec; assertions are explicit conditions.
