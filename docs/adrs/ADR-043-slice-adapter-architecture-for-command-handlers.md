---
id: ADR-043
title: Slice + Adapter Architecture for Command Handlers
status: accepted
features: []
supersedes: []
superseded-by: []
domains:
- api
- testing
- error-handling
scope: cross-cutting
content-hash: sha256:73fb825ad51d0bd3d6fd755c0888ef3d5d5487fa9fa2cbce948e32ef1ac30947
---

**Status:** Proposed

**Context:** Before this decision, CLI command handlers in `src/commands/`
mixed four concerns in a single function: (1) load the knowledge graph,
(2) parse/validate user input, (3) compute the change, (4) write files and
print output. The `handle_status` family reached 334 lines; `handle_adr_write`
reached 227 lines. Mixing concerns caused three problems that recurred across
the codebase.

1. **The business logic was unreachable from unit tests.** Every rule —
   "supersession must not create a cycle", "domain edit must preserve
   vocabulary", "feature abandonment orphans linked tests" — lived inside a
   function that also reached for `std::fs`, `println!`, and the global graph
   loader. Tests had to shell out via `assert_cmd`, spin up tempdirs, and
   parse stdout. A single rule change cost a 200ms integration test instead
   of a 1ms unit test, and the assertion surface was text output rather than
   a typed plan.

2. **Output format was coupled to behaviour.** Adding `--format json` to a
   handler meant finding every `println!`, threading a format flag through
   the handler, and writing a parallel JSON branch. 29 handlers eventually
   needed this and the cost-per-handler discouraged it. Several never got
   JSON output at all.

3. **Error handling was inconsistent.** Some handlers returned
   `BoxResult = Result<(), Box<dyn Error>>`. Others returned
   `Result<(), ProductError>`. Exit codes depended on which path the error
   took through the boxing. A typed error in one handler became an opaque
   boxed error in another, losing the exit-code mapping from
   `error.rs::ProductError::exit_code`.

The root shape: a handler was doing work the domain owned, and the domain
had no home.

---

**Decision:** Organise the codebase as **vertical slices with adapters**.

- A **slice** (`src/<slice>/`) is a pure domain module. It exposes
  `plan_*(state, input) → Plan` functions that take the current graph plus
  user-provided input and return a typed plan struct describing the intended
  change, and `apply_*(plan) → Result<(), ProductError>` functions that
  perform the minimal I/O needed to commit the plan. No `println!`, no
  `std::process::exit`, no format flags, no `Box<dyn Error>`. Renderers
  (`render_*`) turn result structs into text; JSON is derived from
  `serde::Serialize` on the plan types.

- An **adapter** (`src/commands/<cmd>.rs`) is a thin CLI wrapper that loads
  the graph via `shared::load_graph_typed()`, acquires a lock via
  `shared::acquire_write_lock_typed()` when writing, calls the slice's
  `plan_* + apply_*`, and returns
  `CmdResult = Result<Output, ProductError>`. The `render()` bridge in
  `commands/output.rs` turns the `Output` into stdout according to the
  format flag and converts the typed error into a `BoxResult` for legacy
  compatibility with `main.rs`.

- A handler remains on the legacy `BoxResult` signature **only** when it
  has exit-code semantics `CmdResult` cannot express (exit 2 for
  warning-only), prints continuous progress during a long operation, reads
  stdin mid-computation (interactive), or is a trivial printer where
  `Output::Empty` wrapping is pure churn. These criteria are documented in
  `CLAUDE.md`; they are not "legacy" — they are intentional.

---

### 1. The seam — `Output` and `CmdResult`

```rust
pub enum Output {
    Empty,
    Text(String),
    Json(serde_json::Value),
    Both { text: String, json: serde_json::Value },
}
pub type CmdResult = Result<Output, ProductError>;
```

A handler returns `Output::Both { text, json }` from a single pure function.
The `render()` bridge picks the variant based on the `--format` flag. JSON
parity is one line of serialization, not a second code path.

### 2. The atomic-write rule

Multi-file cascades (ADR supersession updating both sides, feature
abandonment orphaning linked tests) call `fileops::write_batch_atomic`
inside the `apply_*` function. Either every file is updated or none is.
This was previously open-coded in adapters with partial-failure hazards;
moving it inside the slice makes it a property of the plan type.

### 3. The unit-test rule

Every slice has a `tests.rs` that exercises the pure `plan_*` functions
against in-memory graphs built from `types::KnowledgeGraph::default()`.
Unit tests do not touch the filesystem, do not call `cargo run`, and do
not parse stdout. A rule change costs one unit test, not a round-trip
through the CLI.

### 4. Adapter size budget

With the slice doing the work, adapters shrink. Observed reductions:
`commands/status.rs` 334 → 68 lines (−80%), `commands/adr_write.rs`
227 → 63 lines (−72%), `commands/adr_seal.rs` 86 → 56 lines (−35%).
The 400-line file cap enforced by
`tests/code_quality_tests.rs::tc_402_*` becomes trivial to satisfy.

### 5. What the adapter still owns

The adapter owns: subcommand definition (clap), graph loading, lock
acquisition, wrapping the slice's return in `Output::Text` or
`Output::Both`. Nothing else. If the adapter needs a conditional or a
computation, the computation belongs in the slice.

---

⟦Γ:Invariants⟧{
  every_slice_function_named_plan_returns_a_typed_plan_without_performing_io
  every_apply_function_takes_a_plan_and_performs_minimal_io_without_computing
  no_slice_function_calls_println_or_eprintln
  no_slice_function_calls_std_process_exit
  every_multi_file_cascade_goes_through_write_batch_atomic_in_one_apply_call
  every_cmdresult_handler_returns_output_not_boxresult
  boxresult_handlers_are_retained_only_for_the_four_documented_criteria
  adapter_size_remains_under_400_lines_after_migration
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

**Rationale:**

- **Separation matches the actual work.** Computing "which files must
  change and what their new contents are" is a pure function of current
  state + input. Writing them is an I/O step with atomicity concerns. These
  are different concerns and benefit from different test harnesses. Pretending
  they are one function just wedges both into a slower integration test.

- **JSON parity is a property of the plan, not the handler.** Once the plan
  is `Serialize`, JSON output costs one line. The previous design made it cost
  a handler-by-handler effort, which is why several handlers silently lacked
  `--format json`. Putting the seam at the handler boundary — `Output::Both`
  — means JSON is always present when the plan is present.

- **Typed errors with documented exit codes.** `CmdResult`'s error arm is
  `ProductError`, which owns the exit-code mapping. `BoxResult` loses this —
  `Box<dyn Error>` exits 1 regardless of the underlying type. Handlers that
  need exit 2 (warning-only, dep-check-style) have a legitimate reason to
  stay on `BoxResult`; handlers that want clean exit-code semantics should
  move to `CmdResult`.

- **File-length and SRP fitness gates compose with this shape.** The 400-line
  file cap and the "no 'and' in module doc comments" SRP check in
  `tests/code_quality_tests.rs` fail gracefully when a handler does too much.
  After migration those checks become trivially satisfied — not because they
  were weakened, but because the work moved to the right place.

- **Incremental migration, not big-bang.** Not every handler needs to move.
  The four BoxResult-retention criteria (exit-code-2, interactive stdin,
  continuous progress, trivial printer) are the genuine exceptions. Migrating
  those would be churn. The rule in `CLAUDE.md` — "migrate only when you have
  a reason: adding JSON parity, extracting for unit test, fixing a bug the
  split makes cleaner" — prevents accidental over-refactoring.

**Rejected alternatives:**

- **Port every handler to `CmdResult`.** Uniform on paper, worse in
  practice. `dep check` returns exit 2 for warning-only state — that is
  load-bearing behaviour callers depend on, not a quirk. `author` prints
  progress during a 30-second operation — wrapping that stream in
  `Output::Empty` drops the progress semantics. "Uniform" here would mean
  adding enum variants to `Output` for each special case until `Output`
  becomes a handler-shape union type; that is a worse design than keeping
  four handlers on the legacy signature and documenting why.

- **Introduce a `Storage` / `Renderer` / `CommandContext` trait stack.**
  Full DIP — every handler takes trait objects. Rejected because the
  codebase has exactly one filesystem and exactly one CLI, and no forseeable
  test needed the indirection that unit-testing the pure slice doesn't
  already give us. The cost of the indirection is paid everywhere; the
  benefit is hypothetical. Revisit when a second backend (MCP over network)
  or a second output (programmatic library mode) actually needs the seam.

- **Keep everything in adapters; extract helpers ad hoc.** The path the
  codebase was already on. Helpers proliferated without a discipline — some
  took `&Graph`, some took `&mut State`, some returned `Result<(), String>`.
  Naming "slice" and "adapter" as explicit layers with contracts (pure /
  I/O) prevents helpers from growing into their own mud ball.

- **Build a framework — a `#[derive(SliceCommand)]` macro that generates
  adapters from slice signatures.** Rejected as premature. The adapter
  boilerplate is ~15 lines. A macro would save typing and cost
  debuggability; the shape has not yet stabilised enough that freezing it
  in a macro is safe. Revisit after a year of this pattern in production.

- **Retroactively split existing `BoxResult` handlers even when none of
  the four criteria apply, for consistency.** Rejected because the
  consistency would be cosmetic. A handler that is 10 lines of
  `println!` on a trivial computation gains nothing from becoming two
  modules plus an adapter. The rule — migrate when you have a reason —
  is the cheap, correct discipline.

**Test coverage:** The pattern is validated at two levels.

1. Per-slice unit tests in `src/<slice>/tests.rs` (feature: 15 tests,
   adr: 21 tests, tc: 7 tests, status: 8 tests) exercise `plan_*`
   functions against in-memory graphs.
2. Code-quality fitness tests in `tests/code_quality_tests.rs` enforce
   the 400-line file cap and the SRP doc-comment check, which are the
   structural evidence that adapters stayed thin and slices stayed
   single-purpose.

No new session tests were written for this ADR — the migration preserves
observable behaviour by construction (394 integration tests + existing
session tests cover the behaviour; 65 new unit tests cover the
refactored pure functions). Future handlers added under this pattern
should gain both integration and session coverage per the usual rule.
