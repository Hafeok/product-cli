---
id: FT-068
title: Convention-Derived TC Runner Config in `product implement`
phase: 5
status: complete
depends-on:
- FT-058
adrs:
- ADR-018
- ADR-021
- ADR-040
- ADR-043
- ADR-047
- ADR-048
tests:
- TC-799
- TC-800
- TC-801
- TC-802
- TC-803
- TC-804
- TC-805
domains: []
patterns:
- PAT-001
- PAT-002
domains-acknowledged:
  ADR-041: Removal & deprecation absence TCs are unrelated. FT-068 does not remove or deprecate any artifact, NuGet/cargo dependency, CLI command, configuration key, or front-matter field. It adds a new pipeline step (Step 0a) and a new flag (`--no-auto-runners`) — pure additions. The existing E022 runner-required gate and the strict invariants from FT-058 stay in place; the auto-fill writes the runner fields the gate demands, it does not remove the gate.
  ADR-049: Per-model context bundle templates are unrelated to this feature. FT-068 modifies the `implement` system prompt (governed by ADR-022) to add a one-line reminder about runner-args; it does not touch context bundle rendering, the `--target` flag, the `[context].default-target` config, or template resolution. The auto-fill operates on TC front-matter directly via the `product test runner` slice, not through any bundle assembly path.
  ADR-042: TC type-system partition (structural reserved types vs. open descriptive types) is unrelated. FT-068 reads TC files purely for their `runner`/`runner-args` front-matter fields and the markdown filename slug; it never branches on the TC `type` field. Both `scenario` TCs and `exit-criteria` TCs go through the same auto-fill code path identically, because the runner-config invariant from FT-058/ADR-021 applies uniformly to every TC linked to an active feature regardless of type.
---

## Description

Move the "auto-fill TC runner config from the TC filename slug" loop
out of harness shell scripts and into `product implement` itself, as
the convention-derived runner-config follow-on that FT-058's
"Out of scope" section explicitly tracked.

Today, every harness that wraps `product implement` has to reinvent
the same pre-flight loop: for each TC linked to the feature, if the
TC lacks `runner` / `runner-args`, derive `tc_<NUM>_<slug>` from the
TC's markdown filename and call `product test runner …` before the
gate fires. Without that loop, `product implement` halts at Step 0
with `E022 — TcRunnerMissing`, the agent never runs, and the
orchestration script's exit semantics get muddled (was the
implementation bad, or was the harness simply missing the auto-fill
loop?).

The strict invariant from FT-058 / ADR-021 stays in place. What
changes is **who** writes the runner-args when the convention is
unambiguous: the CLI, at one well-defined point in the pipeline,
instead of every harness reinventing it.

This is a soft amendment to FT-058's "Out of scope — Convention-
derived runner config" and to ADR-021's runner-config-required
clause. The amendment is narrow: it does **not** weaken any of the
five enforcement gates; it adds a single auto-fill step that runs
*before* `product implement`'s preflight gate and writes runner
fields back to TC front-matter via the existing `product test
runner` slice. By the time the gate evaluates, the runner fields
are present — no gate becomes optional.

---

## Functional Specification

### Inputs

- `FT-XXX` — the feature being implemented, passed to `product
  implement`.
- A `--no-auto-runners` flag (opt-out) on `product implement`.
- The TC markdown filename for each linked TC, used to derive the
  slug.
- The current TC front-matter, to detect which TCs already have
  runner config (no overwrite).

### Outputs

- For each TC missing runner config: a write to the TC's
  front-matter setting `runner: cargo-test`,
  `runner-args: "tc_<NUM>_<slug>"`, `runner-timeout: 120s`.
- A console line per auto-filled TC, identical in spirit to the
  shell-script message:
  ```
  pre-flight: TC-NNN missing runner config — auto-setting
              runner=cargo-test args=tc_NNN_<slug> timeout=120s
  ```
- A trailing summary line: "auto-filled runner config on N TC(s)."
- A reminder to the agent (added to the implement context bundle's
  Hard constraints section): "Test functions must match the
  configured `runner-args` names, or run `product test runner
  TC-XXX --args …` to rename them."

### State

- No new persistent state. The auto-fill writes through the existing
  `product test runner` slice, which already records the change in
  the request log (FT-042 hash-chain).
- A small slice module `src/implement/runner_autofill.rs` exposes:
  - `derive_runner_args(tc_id: &str, tc_path: &Path) -> String`
  - `plan_autofill(graph: &KnowledgeGraph, feature_id: &str)
    -> Vec<AutofillPlan>`
  - `apply_autofill(plans: &[AutofillPlan], root: &Path) -> Result<()>`

### Behaviour

`product implement FT-XXX` gains a new **Step 0a — Auto-fill runner
config** that runs immediately before the existing Step 0 preflight:

1. Enumerate every TC linked to the feature (via `feature.front.tests`).
2. For each TC, read its front-matter. Skip if both `runner` and
   `runner-args` are non-empty.
3. Derive the runner-args slug from the TC filename:
   `tc_<NUM>_<rest_of_filename_with_hyphens_to_underscores>`. This
   matches the convention used in the existing harness loop and the
   `runner-args` hint in the E022 error renderer.
4. Call the existing `product test runner` slice to write
   `runner: cargo-test`, `runner-args: <derived>`,
   `runner-timeout: 120s` to each offending TC.
5. Print one diagnostic line per auto-fill and one summary line.
6. Proceed to Step 0 preflight, which now passes the runner-config
   gate because the fields are populated.

`product implement --no-auto-runners` disables Step 0a entirely.
Useful for CI runs where the operator wants the original strict
gate to fire (e.g. to catch agents that should be declaring
runner-args themselves), and useful for tests that exercise the
E022 path through `product implement`.

`product implement --dry-run` prints what Step 0a *would* write but
does not write. Today `--dry-run` stops before the agent
invocation; the auto-fill step honours the same flag — it shows the
planned writes and continues to Step 0 with the existing (still
unconfigured) graph, exposing whichever gate the user is trying to
preview.

The other four gates (`preflight` invoked directly, `feature status
…in-progress`, `request apply`, `graph check`, `verify`) are
**unchanged**. The auto-fill is local to `product implement`,
because `implement` is the only command that runs the agent and
therefore the only command where the chicken-and-egg friction
exists.

### Invariants

- Step 0a never modifies a TC that already has both `runner` and
  `runner-args` populated. Once an agent or human declares
  runner-args explicitly, the auto-fill leaves it alone forever.
- Step 0a never invents a TC that doesn't exist on disk — it
  iterates `feature.front.tests` and reads the matching files. A TC
  listed in front-matter but missing on disk falls through to the
  existing broken-link error (E002), not auto-fill.
- Step 0a writes through the same `product test runner` code path
  the user would invoke manually, so the request log records the
  write with the existing hash-chain semantics (FT-042).
- The runner-args derivation is a pure function of the TC filename.
  Two invocations of Step 0a on the same graph produce the same
  output (idempotent given the second-skip rule above).
- `--no-auto-runners` restores the pre-FT-068 behaviour exactly:
  Step 0 fails with E022, the agent is not invoked.

### Error handling

- If the TC markdown file is missing on disk, log a warning
  identical to the harness script (`pre-flight: TC-NNN — no
  markdown file under <tests-dir>, skipping`) and proceed. Step 0
  will then fire E022 for that TC, surfacing the underlying
  broken-link problem.
- If the `product test runner` write fails (filesystem error, lock
  contention), bubble the existing error up. No silent swallowing —
  the auto-fill is opportunistic, not load-bearing.
- If the user has set both `runner` and `runner-args` to non-empty
  but garbage values, Step 0a does not touch them. The agent must
  honour the configured names; `product verify` will report
  zero-tests-matched per FT-058.

### Boundaries

- **In scope:** Step 0a auto-fill in `product implement`; the
  `--no-auto-runners` opt-out; the diagnostic output; the implement
  prompt's reminder line; the stale CLAUDE.md "silently skips"
  sentence (correct it to E022); a TC pair that round-trips
  through the new flow.
- **Out of scope:** Auto-fill in any other command. `product
  preflight`, `product graph check`, `product feature status
  …in-progress`, `product request apply`, `product verify` keep
  the strict E022 behaviour. The invariant from FT-058 stands.
- **Out of scope:** Inferring runner type other than `cargo-test`.
  The harness pattern only handles cargo-test today; bash / pytest
  / custom runners require explicit configuration because the test
  identity is not encoded in the TC filename.
- **Out of scope:** Auto-deriving runner-args from the TC `title`
  field rather than the filename. Filenames are stable and
  slug-shaped; titles are free-form. The harness script uses the
  filename for the same reason.
- **Out of scope:** Changing the E022 error message. It already
  prints the canonical YAML snippet that Step 0a writes, so the
  documentation stays consistent.

---

## Out of scope

- **Renaming the gate.** E022 keeps its exit code and rendering.
- **Soft-fail flag on other commands.** Step 0a is `implement`-only;
  the four other gates remain strict.
- **Scanning the test source tree to verify the function exists.**
  That is the existing FT-058 verify-time diagnostic ("No #[test]
  fn matching …"). Step 0a writes the *declared* name; whether the
  function exists is the agent's job to satisfy.
- **Migrating existing TCs.** Every TC in the current corpus
  already has runner config (per the agent-context status), so
  there is no migration to perform.

---

## Implementation notes

- New module `src/implement/runner_autofill.rs`. ~120 lines
  including unit tests. Pure functions only; the apply step
  delegates to the `tc::runner` slice for the actual writes.
- Wire Step 0a into `src/implement/pipeline.rs::run_implement`
  immediately before the existing `Step 0: Preflight` block.
  Threading `--no-auto-runners` flows the same way as the existing
  `--no-verify` / `--dry-run` flags through `commands/implement.rs`.
- Extend the Hard constraints block in the implement prompt
  template (the `format!` on `pipeline.rs:96`) to include the
  reminder line.
- Slug derivation: take the TC filename basename, strip the `.md`
  extension, strip the leading `TC-NNN-` prefix, replace `-` with
  `_`, prepend `tc_<NNN>_`. Matches the harness script verbatim.
- Update `CLAUDE.md` §"TC Runner Configuration": replace "Without
  these fields, `product verify` silently skips the TC" with
  "Without these fields, `product verify` (and four other gates)
  fail with E022; `product implement` auto-fills them from the TC
  filename unless `--no-auto-runners` is set."
- Add a soft amendment record to ADR-021 noting that `product
  implement` ships the auto-fill, citing FT-068.
- File-length: pipeline.rs is currently 181 lines; adding ~30 lines
  for Step 0a wiring keeps it well under the 400-line ceiling. The
  new runner_autofill.rs module sits at ~120 lines.

---

## Acceptance criteria

A developer can:

1. Author a feature with a single TC that has no `runner` field,
   run `product implement FT-XXX`, and observe Step 0a auto-fill
   the TC with `runner: cargo-test`, `runner-args: tc_NNN_<slug>`,
   `runner-timeout: 120s`, followed by a clean Step 0 preflight
   and the rest of the pipeline proceeding.
2. Run the same command with `--no-auto-runners` and observe the
   pre-FT-068 behaviour: E022 fires at Step 0, the agent is not
   invoked, exit code 22.
3. Run `product implement FT-XXX --dry-run` against a feature with
   one unconfigured TC and observe the auto-fill *plan* printed but
   no write occurs (the TC front-matter is unchanged on disk).
4. Author a TC that already has `runner-args: tc_999_custom_name`,
   run `product implement` for its feature, and confirm Step 0a
   leaves it alone (no diagnostic, no write).
5. Run `product feature status FT-XXX in-progress` directly, with a
   TC missing runner config, and observe E022 — the gate fires
   exactly as today, because Step 0a is `implement`-only.
6. Run `product graph check` after manually deleting a `runner`
   line and observe E022 — same as today.
7. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.
