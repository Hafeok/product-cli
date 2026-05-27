---
id: FT-072
title: TC Observability Requirement — observes Field and graph check Validation
phase: 5
status: complete
depends-on:
- FT-003
- FT-018
- FT-038
- FT-048
- FT-069
adrs:
- ADR-051
- ADR-050
- ADR-002
- ADR-013
- ADR-018
- ADR-020
- ADR-042
- ADR-043
- ADR-047
- ADR-049
- ADR-040
- ADR-048
- ADR-041
tests:
- TC-830
- TC-831
- TC-832
- TC-833
- TC-834
- TC-835
- TC-836
- TC-837
- TC-838
domains:
- data-model
- testing
patterns:
- PAT-003
domains-acknowledged: {}
---

## Description

Operationalise ADR-051 by adding the `observes:` front-matter field
to test criteria and validating its presence (for the required tc
types) and its referenced surfaces via `product graph check`. This
feature is the structural fix for the FT-046 → FT-066 anti-pattern:
TCs that pass because they assert on a response envelope instead of
the underlying causation.

The field grammar is intentionally flat (a list of strings) per
ADR-051's "start cheap, promote later" decision. The required-for
set is `scenario | session | smoke | contract`; `invariant`,
`property`, and `chaos` are optional; `exit-criteria` is not
applicable. Existing TCs predating the cutover are grandfathered
via the configurable `[tc-observability].required-from-phase`
threshold.

This feature is independent of the pattern cluster (FT-070, FT-071,
FT-073, FT-074, FT-075) and may be implemented in parallel — its
dependencies are the parser (FT-003), the front-matter mutation
surface (FT-038), and the graph-check infrastructure (FT-018,
FT-069).

---

## Depends on

- **FT-003** — Front-Matter Schema. Owns the parser path that
  accepts the new field.
- **FT-038** — Front-Matter Field Management. Owns the granular
  CLI surface for editing the field.
- **FT-018** — Validation and Graph Health. Owns `product graph
  check`.
- **FT-069** — MCP Parity for `product_graph_check`. The new
  diagnostics must surface over MCP at the same parity FT-069
  established.
- **FT-048** — TC Type System. The required-for matrix uses the
  type values FT-048 standardised.

---

## Functional Specification

### Inputs

- TC YAML front-matter gains `observes:` — a flat list of strings.
- `product.toml` gains:
  - `[tc-observability].required-from-phase` (integer, default
    `5` — grandfathering boundary matching the FT-066 era when the
    lesson was established).
  - `[tc-observability].required-for-types` (list of strings,
    default `["scenario", "session", "smoke", "contract"]`).
  - `[tc-observability].custom` (list of strings; appends to the
    built-in surface vocabulary, mirroring `[tc-types].custom`).
  - `[tc-observability].body-reference-severity` (`warning |
    error`, default `warning` — controls the soft "body lacks
    reference to declared surface" check).
- `product test new --observes file,graph` — extended flag set;
  the scaffolded TC writes the list into front-matter.
- `product test runner` / existing surface — unchanged; the field
  is independent of runner config.
- MCP: `product_test_new` accepts an optional `observes: [String]`
  input; `product_request_apply` accepts `observes:` under the
  `tc` artifact `body` / `field` paths.

### Outputs

- `product graph check`:
  - **Error** (new code, allocated next-free; allocated as `E027`
    pending confirmation at implementation) when a TC of a
    required-for type and phase ≥ threshold has missing or empty
    `observes:`.
  - **Warning** (new code, `W032` pending confirmation) when a
    TC's body text contains no reference to any value in its
    declared `observes:` list (regex check against the surface
    name and a small synonym set).
- `product test show TC-XXX` includes the `observes:` field in
  both text and JSON output.
- `product_request_apply` rejects an `observes:` value that is
  not in the allowed set (built-in + `[tc-observability].custom`)
  with `E026` — the existing strict-validation code from FT-064.

### State

- Extension to `src/types/tc.rs`: `observes: Vec<String>` field
  on `TcFrontMatter` with default empty.
- New module `src/tc/observability.rs`:
  - `pub fn requires_observes(tc_type: &str, phase: i32, config:
    &TcObservabilityConfig) -> bool`
  - `pub fn validate_surface(surface: &str, config: &TcObservabilityConfig)
    -> Result<(), ProductError>`
  - `pub fn body_references_surface(body: &str, surface: &str)
    -> bool`
- Extension to `src/config.rs`: `TcObservabilityConfig` struct
  with the four config keys above.
- Extension to `src/graph/check.rs` (and `full_check.rs` per
  FT-069): two new diagnostics wired into the structural pass.

### Behaviour

1. **Parser.** Accept `observes:` as a flat list of strings on
   any TC. Empty is allowed at parse time; the validation gate
   fires later. Unknown front-matter keys still reject per
   existing schema-strictness rules.

2. **`product test new`.** When the `--observes` flag is passed,
   the scaffolded TC writes the list. When omitted, the scaffold
   leaves the field unset (so the eventual graph-check gate
   fires, prompting the author to declare the surface
   deliberately).

3. **Graph check — required-for gate.** For every TC where:
   - `tc.type ∈ config.required_for_types`,
   - `tc.phase ≥ config.required_from_phase`,
   - `tc.observes` is empty or absent,
   emit the new error code, reference ADR-051, and provide a hint
   listing the allowed surface vocabulary.

4. **Graph check — body-reference gate.** For every TC with
   non-empty `observes:`, scan the body text (case-insensitively)
   for at least one match against each declared surface name (or
   its small synonym set, e.g. `file` matches `file`, `disk`,
   `wrote`, `on-disk`). When zero matches occur across all
   declared surfaces, emit the new warning code. The synonym list
   lives in `src/tc/observability.rs` and is intentionally short
   — the goal is to nudge, not police.

5. **Allowed surface vocabulary.** Built-in: `file`, `graph`,
   `exit-code`, `tag`, `stdout`, `stderr`, `disk-state`,
   `mcp-response`. Extended via `[tc-observability].custom`. A TC
   declaring an unknown surface (built-in nor custom) is rejected
   by `product_request_apply` at write time with E026 (existing
   code).

6. **Grandfathering.** TCs with `phase < required_from_phase`
   are validated identically (parse, list grammar) but exempt
   from the missing-`observes:` gate. The default threshold of
   phase 5 grandfathers every TC in the current corpus.

7. **AISP-block guidance.** The implement prompt template and
   `author-feature` system prompt are updated to reference
   ADR-051 explicitly. The TC scaffolding for any feature
   authored after this lands carries `observes:` declarations
   from day one.

8. **MCP parity.** Both new diagnostics flow through
   `graph::full_check::run` (FT-069), so MCP and CLI surface
   identical findings; the FT-069 parity invariant holds.

### Invariants

- Every TC of a required-for type at phase ≥ threshold carries a
  non-empty `observes:` list, or `product graph check` reports an
  error.
- Every value in any TC's `observes:` list is a recognised
  surface (built-in or `[tc-observability].custom`).
- `product graph check` MCP and CLI JSON envelopes are
  byte-identical for the new diagnostics (FT-069 parity
  generalised).
- The required-for set, phase threshold, and custom vocabulary
  are configurable; defaults match ADR-051.
- The body-reference check is a warning by default and never
  blocks merge in absence of explicit promotion via
  `[tc-observability].body-reference-severity = "error"`.
- Existing TCs (phase < threshold) are not affected by the new
  gates beyond grammar validation.

### Error handling

- **E027 (pending; allocate next-free at implementation)** — TC
  of required-for type missing `observes:`. Renders as:
  ```
  error[E027]: TC missing required observes field
    --> docs/tests/TC-XXX-foo.md
     = TC-XXX (type: scenario, phase: 5) declares no observable
       surface, but its type is in
       [tc-observability].required-for-types
     = hint: add a non-empty `observes:` list to the front-matter
       (allowed: file, graph, exit-code, tag, stdout, stderr,
        disk-state, mcp-response)
     = see ADR-051
  ```
- **W032 (pending; allocate next-free)** — TC body lacks
  reference to any declared surface. Warning tier; suppressible
  by adding the reference to the body. Renders similarly with
  a hint pointing at the missing surface.
- **E026** — Unknown surface value in `observes:`. Reuses the
  existing strict-validation code from FT-064.
- **E001** — Malformed YAML on the new field (not a flat list of
  strings). Existing parse-error path.

### Boundaries

- **In scope:** the `observes:` field, the parser extension, the
  `product test new --observes` flag, the two new graph-check
  diagnostics, the config keys, MCP parity for both diagnostics,
  the small synonym set for the body-reference check, updates to
  the implement and author-feature prompts referencing ADR-051,
  documentation in AGENTS.md and the schema MCP tool.
- **Out of scope:** the structured-object form of `observes:`
  (`[{kind: file, path: "..."}]`) — deferred per ADR-051's
  "start flat, promote later" decision.
- **Out of scope:** mutation testing as a backstop (deferred per
  ADR-051's rationale).
- **Out of scope:** automatic migration of existing TCs to add
  `observes:` — grandfathering covers this without intervention.
- **Out of scope:** the alone-`mcp-response` fitness function
  (the "observes only the response envelope" anti-pattern check).
  ADR-051 names this as a follow-on; it would land as a separate
  feature once the field-level checks have settled.

---

## Out of scope

- All items under "Boundaries → Out of scope" above.
- Changes to existing TC types or the type system itself
  (FT-048 / ADR-042 still own that).
- Schema validation of TC `observes:` against the actual
  assertion library used in the test source — too codebase-
  specific; out of scope for a structural check.
- Cross-cutting "observes" semantics for ADRs or features
  (ADR-051 is explicit that only TCs carry the field).

---

## Implementation notes

- **`src/tc/observability.rs`** — pure module, ~150 lines
  including tests. Owns the surface vocabulary, the
  required-for predicate, the body-reference check, and the
  synonym list. Slice-style — no I/O, no println.
- **`src/types/tc.rs`** — add `observes: Vec<String>` with
  default empty. Serialisation: when the list is empty, omit the
  key from the YAML (matches the `omit_if_empty` pattern used
  elsewhere).
- **`src/config.rs`** — add `TcObservabilityConfig`. Default
  values literal in the struct so config-free repos behave per
  ADR-051's defaults.
- **`src/graph/check.rs`** and `src/graph/full_check.rs` — add
  two new check functions; register them in the structural
  pass. Both find-and-report; no graph mutation.
- **`src/commands/tc_write.rs`** — extend `product test new` to
  accept `--observes file,graph` (comma-separated) and pass the
  list through to the slice.
- **`src/request/validate.rs`** — validate `observes:` values
  against the configured vocabulary; emit E026 on unknown.
- **`src/mcp/tools/`** — extend `product_test_new` schema with
  the optional `observes` field.
- **Prompt updates** — `docs/prompts/implement-v1.md` gains a
  bullet "Every TC under test must declare `observes:`;
  assertions must target the named surface(s)". The
  `author-feature-v1.md` and `author-review-v1.md` prompts gain
  similar bullets and reference ADR-051.
- **Schema docs** — the agent-context generator (`product
  agent-init` output) lists the new field and the allowed
  vocabulary.
- **AGENTS.md** — the working protocol section adds a note about
  the new gate.
- **File-length budget:** `src/tc/observability.rs` stays under
  200 lines. The graph-check additions are two ~30-line
  functions.
- **Concurrency:** read-side / validation only; no new locking.

---

## Acceptance criteria

A developer can:

1. Run `product test new --type scenario --observes file,graph
   "test_name"` and observe a TC scaffolded with `observes:
   [file, graph]` in the front-matter.
2. Run `product graph check` against a repo with a phase-5
   scenario TC lacking `observes:` and observe the new error
   code citing the missing field, with ADR-051 referenced.
3. Run `product graph check` against a repo with a TC declaring
   `observes: [file]` whose body contains no reference to file
   writes; observe the new warning code.
4. Apply a request adding `observes: [unknown_surface]` to a TC
   via `product_request_apply`; observe E026 rejection naming the
   unknown surface and listing the allowed vocabulary.
5. Verify that the existing TC corpus (phase < 5 TCs and
   pre-FT-072 phase 5 TCs) passes `product graph check` without
   regression — grandfathering works as configured.
6. Confirm the same diagnostics appear over MCP via
   `product_graph_check` (FT-069 parity invariant).
7. Update `[tc-observability].required-from-phase = 1` and
   observe every phase-1 TC of a required-for type that lacks
   `observes:` flagged; revert and confirm grandfathering
   restores.
8. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

---

## TC scaffolding plan

Every TC scaffolded for this feature itself dogfoods the new
field — the F3 implementation cannot ship without F3-compliant
test criteria.

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `tc_observes_field_parses_as_flat_list` | scenario | `[file, graph]` | Parser accepts `observes: [file, graph]` and round-trips through write. |
| `tc_observes_missing_on_required_type_emits_error` | scenario | `[stdout, exit-code]` | Graph check exits non-zero with the new error code naming the offending TC. |
| `tc_observes_missing_on_optional_type_passes` | scenario | `[exit-code]` | Invariant / property TCs without `observes:` do not trip the gate. |
| `tc_observes_body_lacking_reference_emits_warning` | scenario | `[stdout]` | A TC declaring `observes: [file]` whose body never mentions file writes emits W-tier warning. |
| `tc_observes_unknown_surface_rejected_by_request_apply` | scenario | `[stdout, exit-code]` | `product_request_apply` rejects `observes: [bogus]` with E026. |
| `tc_observes_custom_surface_accepted_via_config` | scenario | `[file]` | Adding `[tc-observability].custom = ["custom_surface"]` makes the value accepted. |
| `tc_observes_grandfathering_threshold_works` | scenario | `[stdout]` | Setting `required-from-phase = 99` exempts all phase-5 TCs; setting it to `1` flags them. |
| `mcp_graph_check_observes_findings_match_cli_json` | scenario | `[mcp-response, stdout]` | MCP JSON envelope for `product_graph_check` byte-equals CLI `--format json` for a fixture exercising the new diagnostics. |
| `ft_072_exit_criteria_observes_field` | exit-criteria | n/a | Aggregator; cargo gates green; the implement and author-feature prompts reference ADR-051. |
