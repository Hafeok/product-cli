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
domains:
- api
- error-handling
domains-acknowledged: {}
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
