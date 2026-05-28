---
id: FT-047
title: Removal & Deprecation Tracking — Absence TCs and ADR Lifecycle Fields
phase: 5
status: complete
depends-on:
- FT-018
- FT-029
adrs:
- ADR-002
- ADR-013
- ADR-019
- ADR-040
- ADR-041
- ADR-042
tests:
- TC-586
- TC-587
- TC-588
- TC-589
- TC-590
- TC-591
- TC-592
- TC-593
- TC-594
- TC-595
- TC-596
- TC-597
- TC-598
- TC-599
- TC-600
domains:
- api
- data-model
- error-handling
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
---

## Description

Product is a knowledge graph for *what was decided*. Half of every non-trivial
decision is "and the old thing goes away". This feature delivers the spec-layer
machinery to express, link, and verify removal and deprecation: a new TC type
(`absence`), two new ADR front-matter fields (`removes` and `deprecates`), and
three new validation codes (G009, W022, W023).

The full design is captured in `docs/product-removal-deprecation-spec.md`. This
feature implements that spec.

---

## Depends on

- **FT-018** — Validation and Graph Health. W022 and W023 surface through the
  existing E/W code stream. This feature extends the vocabulary; the surfacing
  machinery already exists.
- **FT-029** — Gap Analysis. G009 surfaces through the existing structural gap
  check stream. After ADR-040, gap check is structural-only; G009 fits cleanly
  into that bucket (no LLM required — pure front-matter shape check).

---

## Scope of this feature

### In

1. **`tc-type: absence`** — extend the TC enum and the request schema. An
   absence TC is structurally identical to a scenario TC except its
   `validates.features` is required to be empty and `validates.adrs` must be
   non-empty. Validation rejects misshapen absence TCs with a clear error.
2. **`removes` and `deprecates` ADR fields** — both default to empty array,
   both are arrays of freeform strings. Parser, serialiser, schema doc, and
   agent-context schema render are all updated.
3. **G009** — `product gap check` reports G009 for any ADR whose `removes` or
   `deprecates` is non-empty and whose linked TCs include none of `tc-type:
   absence`. Severity: high.
4. **W022** — `product graph check` reports W022 for the same condition. The
   shared underlying check lives once in the validator; the two surfaces both
   render it.
5. **W023** — when the front-matter parser encounters a field that is named in
   the `deprecates:` list of any accepted ADR, emit W023 at parse time. The
   field is still parsed and the graph still builds. The warning names the
   deprecating ADR.
6. **Platform verify integration** — `product verify --platform` (stage 6 of
   the unified verify pipeline, ADR-040) discovers absence TCs by `tc-type` and
   runs them. No changes to the runner machinery.
7. **Schema doc / agent-context updates** — the front-matter schema rendered by
   `product agent-context` and `product_schema` documents the new fields and
   the new `absence` TC type.
8. **Runner pattern reference** — the standalone spec
   `docs/product-removal-deprecation-spec.md` already documents seven runner
   patterns (removed CLI, removed NuGet/npm/cargo, file absent, deprecation
   warning emitted). This feature ships with the spec; no additional reference
   scripts are required to live in the repository.

### Out

- **Auto-generation of absence TCs from `removes` strings.** The strings are
  freeform and ecosystem-specific. Generating a TC requires a runner script,
  which requires the user to know the runtime. Out of scope; the user writes
  the runner.
- **Built-in runner library for ecosystem-specific patterns.** The spec
  documents seven patterns; shipping them as opinionated templates pulls
  ecosystem knowledge into Product. The runner remains the user's
  responsibility.
- **Model of *what* is removed.** Product does not parse `removes` strings or
  attempt to relate them to dependencies (DEPs), source files, or CLI command
  registries. The string is a label, not a query.
- **Deprecation of the `removes`/`deprecates` fields themselves.** Self-host
  is not in scope. A future ADR may invoke the `deprecates` machinery on its
  own predecessor — that is fine and the recursion is well-defined.
- **Auto-promotion of phase-1 deprecation TCs to `unrunnable` once the phase-2
  absence TC passes.** Manual transition with a documented reason. Automating
  it conflates two distinct decisions (the migration is complete *and* the
  author wants to retire the warning TC).

---

## Commands

No new CLI subcommands. The feature surfaces entirely through:

- The existing `product graph check` (W022, W023).
- The existing `product gap check` (G009).
- The existing `product verify --platform` (running absence TCs).
- The existing `product request {validate,apply}` (creating absence TCs and
  ADRs with `removes`/`deprecates`).

---

## Implementation notes

- **`src/types.rs`** — extend `TcType` enum with `Absence`. Update
  `Display`/`FromStr`/serde representations. Add `removes: Vec<String>` and
  `deprecates: Vec<String>` to the `Adr` struct, defaulting to empty.
- **`src/parser.rs`** — accept the new TC type value. Accept the two new ADR
  fields. The parser must not reject older ADRs without these fields (default
  to empty). Round-trip serialise emits the fields only when non-empty (avoid
  churn in existing files).
- **`src/graph.rs`** — extend graph-check rules. New rule
  `check_removes_has_absence_tc` runs over every accepted ADR; emits W022 when
  violated. New rule `check_deprecated_field_usage` runs at parse time, builds
  the union of `deprecates:` strings across all accepted ADRs, then reports
  W023 for each artifact whose front-matter contains a key in that set.
- **`src/gap.rs`** — `check_structural` gains a G009 emitter. Same condition
  as W022, different code, high severity. Returns a `Finding` per offending
  ADR.
- **`src/verify/platform.rs`** (or wherever stage-6 of the verify pipeline
  lives) — when collecting platform TCs, include any TC with `tc-type:
  absence`. The selection rule for platform TCs is already
  "validates.features is empty AND validates.adrs is non-empty"; absence TCs
  match this naturally. The change is to pass `tc-type` through to the runner
  for diagnostic purposes.
- **`src/request.rs`** — request schema validators accept the new TC type and
  the new ADR fields. Cross-artifact validation rejects absence TCs whose
  `validates.features` is non-empty (E006: invalid shape for tc-type absence).
- **`src/domains.rs`** — no changes; the new fields are not domain-related.
- **Tests.** Each TC under FT-047 is implemented as an integration test under
  `tests/integration.rs` or a session test under `tests/sessions/` per FT-043
  conventions, paired with a `runner: cargo-test` configuration whose
  `runner-args` matches the test function name in the conventional
  `tc_NNN_snake_case` form. Runner config is added at the same time as the
  test (CLAUDE.md rule).

---

## Acceptance criteria

A developer running on a clean repository can:

1. Create a TC with `tc-type: absence` via `product request apply` and observe
   the file is written, the TC appears in the graph, and `product verify
   --platform` runs its runner (TC-586, TC-588).
2. Run an absence TC whose runner exits non-zero and observe the TC is marked
   `failing` and the platform verify exits 1 (TC-587).
3. Author an ADR with non-empty `removes` and observe the value round-trips
   through parse/serialise (TC-589). Same for `deprecates` (TC-590).
4. Run `product gap check` against an ADR with `removes` but no linked
   absence TC and observe G009 in the output, severity high (TC-591).
5. Run `product graph check` against the same ADR and observe W022 in the
   warning stream (TC-592).
6. Link an absence TC to the ADR via `product request apply` and observe both
   G009 and W022 disappear (TC-593).
7. Author an ADR that deprecates a front-matter field, accept it, then load a
   repository that uses that field and observe W023 emitted at parse time
   (TC-594). Confirm the field is still present in the parsed artifact and
   the graph builds normally (TC-595). Confirm the W023 message names the
   deprecating ADR by ID (TC-596).
8. Author the phase-1 (deprecation warning emitted) absence TC and confirm it
   passes when the warning is observed (TC-597).
9. Author the phase-2 (thing absent) absence TC and confirm it passes when
   the thing is removed (TC-598). Mark the phase-1 TC `unrunnable` with a
   reason and confirm the change does not block CI (TC-599).
10. Run `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
    and `cargo build` and observe all pass.

See TC-600 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Auto-generated `tc-type: absence` scaffolding.** A future feature may
  scaffold a TC stub from each `removes` entry on ADR acceptance, with the
  runner field left empty for the user to fill in. Useful but not required;
  G009 already drives the work.
- **Cross-language runner library.** Optional curated bash scripts under
  `scripts/test-harness/` mirroring the spec's seven patterns. Useful as a
  starting point for new repos; not required for the spec layer to be sound.
- **Self-host deprecation example.** A real ADR on Product itself that uses
  `deprecates: [source-files]` to retire the long-deprecated `source-files`
  front-matter field. Demonstrates the machinery; out of scope for this
  feature.

---

## Functional Specification

### Inputs

- TC documents with `tc-type: absence` in their YAML front-matter — created via `product request apply` or authored directly. An absence TC requires `validates.features` to be empty and `validates.adrs` to be non-empty.
- ADR documents with `removes: [String]` and/or `deprecates: [String]` arrays in their YAML front-matter — new optional fields, defaulting to empty; round-tripped by the parser/serialiser without churn on existing ADRs that omit them.
- `product gap check` — reads the knowledge graph; evaluates the structural G009 rule.
- `product graph check` — reads the knowledge graph; evaluates the structural W022 and W023 rules.
- `product verify --platform` — discovers TCs by `tc-type` (including `absence`) for stage-6 execution.
- `product request {validate,apply}` — validates TC type and absence TC shape as part of the request validation pipeline.
- `product agent-context` / `product_schema` — reads graph and config; renders the updated schema.

### Outputs

- **`product gap check`** — G009 finding (severity: high) for any ADR whose `removes` or `deprecates` is non-empty and whose linked TCs include no `tc-type: absence`.
- **`product graph check`** — W022 finding for the same condition as G009; W023 finding for each artifact whose front-matter contains a key listed in the `deprecates:` array of any accepted ADR.
- **`product verify --platform`** — runs `tc-type: absence` TCs alongside other platform TCs; per-TC result (pass / fail / unrunnable) contributed to the stage-6 result.
- **`product request validate`** — E006 for absence TCs whose `validates.features` is non-empty (invalid shape); E006 for unknown `tc-type` values.
- **`product agent-context` / `product_schema`** — updated schema documenting the `absence` TC type, `removes`, and `deprecates` ADR fields.
- **W023 at parse time** — when the parser encounters a front-matter field that is listed in the `deprecates:` array of any accepted ADR; the field is still parsed and the graph builds normally.

### State

- **`src/types.rs` `TcType` enum** — extended with the `Absence` variant. The serialised spelling is `"absence"`. No migration of existing TCs is required; the parser default for unrecognised types is an E006 finding (unchanged).
- **`Adr` struct** — extended with `removes: Vec<String>` and `deprecates: Vec<String>`, both defaulting to empty. Serialiser emits these fields only when non-empty (avoids churn in existing ADR files on round-trip).
- No new files, no new graph store, no new persistent state beyond what is authored in existing YAML front-matter.

### Behaviour

1. **`tc-type: absence` validation.** The parser and request validator both accept `"absence"` as a valid TC type. Structural validation additionally checks that `validates.features` is empty (E006 if non-empty) and `validates.adrs` is non-empty. The runner machinery in `product verify --platform` discovers absence TCs by the same selection rule as other platform TCs (`validates.features` empty, `validates.adrs` non-empty) and runs their configured runner.
2. **`removes` and `deprecates` round-trip.** The parser accepts both fields on ADR front-matter; the serialiser emits them only when non-empty. Older ADR files without these fields parse without error (default empty).
3. **G009 in `product gap check`.** For each accepted ADR: if `removes` or `deprecates` is non-empty and no linked TC has `tc-type: absence`, emit G009 (severity: high). G009 is structural — no LLM required.
4. **W022 in `product graph check`.** Same condition as G009, emitted as a W-class warning through the graph-check stream.
5. **W023 at parse time.** At graph-build time, the validator constructs the union of all `deprecates` string values across accepted ADRs. For each artifact, it checks whether any front-matter key appears in that union; matching keys emit W023 naming the deprecating ADR by ID. The artifact is still included in the graph; W023 is advisory.
6. **Schema and agent-context updates.** The `absence` TC type, `removes`, and `deprecates` fields are documented in the schema rendered by `product agent-context` and `product_schema`, grouped appropriately (structural type for `absence`; optional array fields for `removes`/`deprecates`).

### Invariants

- An absence TC with non-empty `validates.features` is rejected with E006 by both the request validator and `product graph check`. It is never written to disk via the request interface.
- W023 does not prevent the graph from building; it is purely advisory. Any artifact with a deprecated field is still parsed and included.
- G009 and W022 disappear when at least one `tc-type: absence` TC is linked to the ADR whose `removes` or `deprecates` triggered the finding. Verified by TC-593.
- The `removes` and `deprecates` fields are emitted by the serialiser only when non-empty, so existing ADR files are not modified on round-trip.
- `product verify --platform` (stage 6 of the unified verify pipeline) discovers absence TCs by the same selection rule as other platform TCs; no special case is needed.

### Error handling

| Code | Tool | Condition |
|---|---|---|
| G009 | `product gap check` | ADR has non-empty `removes` or `deprecates` but no linked `tc-type: absence` TC; severity: high |
| W022 | `product graph check` | Same condition as G009; emitted as a warning in the graph-check stream |
| W023 | Parser / `product graph check` | Artifact front-matter contains a key listed in the `deprecates:` array of an accepted ADR; advisory, graph builds normally |
| E006 | Request validator / `product graph check` | Absence TC with non-empty `validates.features`, or unknown `tc-type` value |

### Boundaries

- Auto-generation of absence TC scaffolding from `removes` entries is not implemented; G009 drives the work and the user writes the runner.
- No built-in runner library for ecosystem-specific absence patterns is shipped; the spec documents seven runner patterns but they remain the user's responsibility.
- Product does not parse `removes` or `deprecates` strings as structured identifiers; they are freeform labels — not queries against DEPs, source files, or CLI command registries.
- Auto-promotion of phase-1 deprecation TCs to `unrunnable` once the phase-2 absence TC passes is not implemented; manual transition with a documented reason is required.

## Out of scope

- Auto-generation of absence TC stubs from `removes` entries: G009 already drives the work; scaffolding is a future feature.
- Built-in runner library for ecosystem-specific patterns (removed CLI, removed NuGet/npm/cargo, file absent): the spec documents seven runner patterns; shipping them as templates pulls ecosystem knowledge into Product.
- Structured modelling of `removes` strings: the values are freeform labels, not queries against DEPs, source files, or CLI registries.
- Deprecation of the `removes`/`deprecates` fields themselves: self-hosting via the `deprecates` field on a future ADR is well-defined but out of scope for this feature.
- Auto-promotion of phase-1 deprecation TCs to `unrunnable` once the phase-2 absence TC passes: this conflates two distinct author decisions and is left as a manual step.
