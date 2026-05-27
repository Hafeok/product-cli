---
id: FT-075
title: Seed Pattern Catalog — PAT-001 Slice + Adapter, PAT-002 MCP Tool With Disk Side-Effect, PAT-003 TC Observability
phase: 5
status: complete
depends-on:
- FT-070
- FT-071
- FT-072
- FT-073
adrs:
- ADR-050
- ADR-051
- ADR-043
- ADR-020
- ADR-038
- ADR-018
- ADR-048
- ADR-049
- ADR-042
- ADR-041
- ADR-047
- ADR-040
tests:
- TC-854
- TC-855
- TC-856
- TC-857
- TC-858
- TC-859
domains:
- data-model
- testing
domains-acknowledged: {}
---

## Description

Seed the pattern catalog with three concrete PATs that capture
patterns already living informally in CLAUDE.md and oral
tradition. The three seeds serve two purposes:

1. **Bootstrap.** Future features have something to cite from day
   one. Without seeds, the F4 advisory warning ("feature cites no
   pattern") has nothing to suggest and the F5 implement bundle
   has no patterns to surface.
2. **Dogfood test for F1.** If any of the three patterns needs a
   front-matter field or body section ADR-050 / F1 do not
   provide, the schema is incomplete. Fix F1 before F6 ships.

The three seeds are exactly the ones called out in the originating
brief:

- **PAT-001 — Slice + Adapter module structure** (from
  `CLAUDE.md` "Architecture Pattern" section; codified in
  ADR-043).
- **PAT-002 — MCP tool with disk side-effect** (the
  FT-046 → FT-066 lesson; codified in ADR-020 and FT-066's TC
  shape).
- **PAT-003 — TC authoring: observability and causation** (the
  ADR-051 contract; the discipline FT-072 enforces).

Each seed is authored through the F4 `author-pattern` flow (or
directly via `product_request_apply`) and lands in
`docs/patterns/`. Each cites its governing ADR(s), declares
prerequisites (where applicable), and lists worked-example
features that exemplify the pattern in the existing codebase.

---

## Depends on

- **FT-070** — Pattern Artifact. Required to write the PAT
  files at all.
- **FT-071** — Pattern in Graph Algorithms. Required to surface
  the seeds in `product context`, `product impact`, and
  centrality. Without F2 the seeds exist but cannot be read by
  any consumer.
- **FT-073** — Pattern Authoring. The dogfood path. F6 prefers
  authoring through `product author pattern` over a raw request
  YAML; if F4 lands first this is straightforward, if not the
  seeds can still be applied via `product_request_apply`
  directly (the request interface ships in F1).
- **FT-072** — TC Observability. PAT-003's body cannot be
  written until the field exists in the schema F3 ships;
  authoring it earlier would require placeholder content.

---

## Functional Specification

### Inputs

- ADR-043 (Slice + Adapter), ADR-020 (MCP dual transport),
  ADR-051 (TC observability) — the ADRs the seeds
  operationalise.
- Existing features that exemplify the patterns in real code:
  FT-066, FT-068, FT-069 (slice + adapter); FT-066, FT-069 (MCP
  with disk side-effect); FT-066 TC-778..TC-784 (the TC
  observability discipline as it was first practised).

### Outputs

- Three new files on disk:
  - `docs/patterns/PAT-001-slice-adapter-module-structure.md`
  - `docs/patterns/PAT-002-mcp-tool-with-disk-side-effect.md`
  - `docs/patterns/PAT-003-tc-authoring-observability-and-causation.md`
- Reciprocal `examples:` ↔ `feature.patterns:` writes on every
  cited example feature (FT-066, FT-068, FT-069 each gain
  entries in their `patterns:` array; the actual list depends on
  which features exemplify which pattern — see Behaviour).
- All three pass `product graph check` cleanly: no
  requires-cycle, no missing body section, no deprecated-pattern
  warning.

### State

- Three pattern files on disk.
- Updates to `FT-066.patterns`, `FT-068.patterns`,
  `FT-069.patterns` (and any other example feature touched).
- The request log records the seed batch as one or more atomic
  applies.
- No other persistent state.

### Behaviour

1. **PAT-001 — Slice + Adapter module structure.**
   - `status: live`
   - `domains: [api, data-model]`
   - `adrs: [ADR-043]`
   - `requires: []` (no prerequisite)
   - `examples: [FT-066, FT-068, FT-069]`
   - Body sections:
     - **When to use:** "Any new CLI command (or MCP tool) that
       does non-trivial work needs unit-testable business logic."
     - **Prerequisites:** Familiarity with `ProductError` and the
       atomic-write helpers in `fileops`.
     - **The pattern:** Concrete code sketch (mirroring ADR-043's
       structure) showing `plan_*` (pure) + `apply_*` (I/O) +
       `render_*` (text) split, with the adapter thin layer
       returning `CmdResult`.
     - **Anti-patterns:** "Doing the work in `commands/foo.rs`
       directly so the slice has nothing to test"; "Returning
       `BoxResult` from a new handler when no exception in
       CLAUDE.md applies"; "Calling `println!` from inside the
       slice".
     - **Worked example:** References FT-066's
       `src/feature/link.rs` (the cleanest current example).
2. **PAT-002 — MCP tool with disk side-effect.**
   - `status: live`
   - `domains: [api, storage]`
   - `adrs: [ADR-020, ADR-038]`
   - `requires: [PAT-001]` (depends on the slice + adapter
     pattern)
   - `examples: [FT-066, FT-068, FT-069]`
   - Body sections:
     - **When to use:** "Any MCP tool whose contract advertises a
       write (`requires_write: true`) must produce a corresponding
       on-disk effect."
     - **Prerequisites:** PAT-001 (the slice the tool dispatches
       into).
     - **The pattern:** The MCP handler is a thin call to
       `slice::plan_*` + `apply_*`. The slice owns the write
       through `fileops::write_file_atomic` or
       `write_batch_atomic`. Includes a code sketch.
     - **Anti-patterns:** "Returning a success envelope from a
       no-op stub" (the FT-046 → FT-066 case study, named
       explicitly); "Routing two distinct tools through one
       shared handler that discards the type information";
       "Adding a `note: \"Use CLI for ...\"` field to a
       supposedly-equivalent MCP write".
     - **Worked example:** References FT-066 — both the bug and
       the fix — and the TC-778 / TC-787 assertion shape.
3. **PAT-003 — TC authoring: observability and causation.**
   - `status: live`
   - `domains: [testing]`
   - `adrs: [ADR-051]`
   - `requires: []`
   - `examples: [FT-066, FT-072]` (FT-072 is the feature that
     authored this contract structurally; FT-066 is the original
     case study)
   - Body sections:
     - **When to use:** "Every TC of type `scenario`, `session`,
       `smoke`, or `contract` from phase 5 onward."
     - **Prerequisites:** None.
     - **The pattern:** Declare `observes:` explicitly. The body
       asserts on the named surface (a file on disk, a graph
       node, an exit code) — not on the return-type shape alone.
       Includes a code sketch of the TC body for a
       `scenario / observes: [file, mcp-response]` case mirroring
       FT-066's TC-778.
     - **Anti-patterns:** "TC asserts on `Ok(_)` shape only";
       "TC observes only the MCP response envelope without
       inspecting the file the response claims to have written"
       (the FT-046 shape); "TC declares `observes: [file]` but
       its body never reads the file".
     - **Worked example:** References FT-066's TC-778, TC-779,
       TC-787 (the post-fix TC family).

4. **Authoring path.** Each seed is authored via
   `product author pattern` when F4 has landed. If F6 is being
   applied before F4 in some interleaved schedule, the same seeds
   can be applied through `product_request_apply` with a single
   atomic batch — both paths satisfy F1's CRUD contract.

5. **Reciprocation.** Each seed's `examples:` list triggers
   FT-070's bidirectional write — every example feature's
   `patterns:` array is updated in the same batch.

6. **Validation.** After applying the seed batch:
   - `product graph check` must exit 0 (no errors, no warnings
     introduced).
   - `product pattern list` shows all three.
   - `product impact PAT-001` lists FT-066, FT-068, FT-069.
   - `product context FT-066 --depth 1` includes PAT-001,
     PAT-002, PAT-003 (depending on the actual catalog
     reciprocation) in topo order.
   - The bundle metadata `FT-066.bundle.patterns` reads ≥ 1
     after `--measure`.

### Invariants

- All three seeds parse successfully (round-trips through the
  parser and writer).
- All three seeds pass FT-071's body-section check (every
  required H2 present).
- The `requires:` chain (`PAT-002 → PAT-001`) is acyclic.
- Every `examples:` reference resolves to a real feature id;
  every reciprocal `feature.patterns:` write lands.
- `product graph check` exits 0 after the seed batch.
- The three seeds, viewed together, demonstrate every front-matter
  field ADR-050 defines (the schema-completeness dogfood test —
  if a field cannot be exercised by these three, the schema is
  over-broad).

### Error handling

- Any seed application that fails per F1 / F2's existing error
  surfaces (E001, E002, E003, E026, W031) blocks the F6 batch
  atomically — the request interface does not partially apply.
- A failure to write reciprocal `feature.patterns:` is treated
  identically to FT-066's reciprocation invariant.

### Boundaries

- **In scope:** the three seeds, their `examples:`
  reciprocation, the validation that the catalog passes
  `graph check`.
- **Out of scope:** more than three seeds. Additional patterns
  accrete as opportunities arise; this feature is the floor, not
  the ceiling.
- **Out of scope:** wholesale migration of CLAUDE.md content
  into patterns. Per the brief: "rest accretes as opportunities
  arise".
- **Out of scope:** patterns for any other ADR not already in
  this list.
- **Out of scope:** patterns capturing planning, project
  management, or process — these seeds are strictly
  implementation patterns.

---

## Out of scope

- Additional seed patterns beyond the three named.
- Cross-project pattern publishing.
- Pattern templates that aren't backed by an existing ADR.
- Generated documentation guides for patterns (Diátaxis
  treatment; FT-070 stays as the canonical home).

---

## Implementation notes

- **Authoring approach.** Run `product author pattern` three
  times against the live repo after F1–F4 have landed.
  Alternatively, prepare a single YAML request and apply via
  `product_request_apply` for a clean atomic write recorded as
  one hash-chain entry.
- **Body content.** Each pattern's `## The pattern` section
  includes a short Rust code snippet (or a structural sketch)
  pulled from the cited example feature. Keep snippets under
  20 lines each; reference the real file path for the full
  context.
- **Anti-pattern naming.** Each anti-pattern is a named case
  with one-sentence cost ("returns success without disk write,
  agents cannot distinguish from real success"). Naming is
  load-bearing — F4's "propose matching patterns" surfaces the
  pattern title; the body provides the reasoning.
- **Reciprocation.** The example features (FT-066, FT-068,
  FT-069) gain `patterns:` entries in front-matter. If those
  features are at `status: complete` (they are), the writes are
  still permitted — `patterns:` is a metadata-edit, not a
  state-change, and `feature link --pattern` (FT-073) does not
  require the feature to be in any particular status.
- **File-length budget:** each seed pattern file targets ≤ 250
  lines including code snippets.
- **Validation script:** add a session test
  (`tests/sessions/ft_075_seed_pattern_catalog.rs`) that
  composes a fresh temp repo, applies the seed batch via
  `product_request_apply`, and asserts every invariant from the
  Functional Specification.

---

## Acceptance criteria

A developer can:

1. After F6 lands, list patterns and observe all three seeds in
   `product pattern list`.
2. Read each seed file and confirm it has every required H2
   body section non-empty.
3. Run `product graph check` and observe exit 0 (no warnings,
   no errors introduced by the seeds).
4. Run `product impact PAT-001` and observe FT-066, FT-068,
   FT-069 (or whichever example list each seed cites) in the
   impact tree.
5. Run `product context FT-066 --depth 1` and observe PAT-001,
   PAT-002, PAT-003 rendered in topo order under "## Patterns".
6. Run `product graph central --include patterns` and observe
   the three seeds appearing in the centrality ranking (the
   three are the only patterns at this point, so their
   centrality is whatever the graph affords).
7. Confirm that PAT-002's `requires: [PAT-001]` chain renders
   correctly in `product context` (PAT-001 before PAT-002).
8. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

---

## TC scaffolding plan

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `seed_catalog_three_patterns_parse_and_validate` | scenario | `[file, graph, exit-code]` | After applying the seed batch, all three pattern files parse, the graph exposes them, and `graph check` exits 0. |
| `seed_pat_002_requires_pat_001_topo_visible_in_context` | scenario | `[stdout]` | `product context FT-066 --depth 1` renders PAT-001 before PAT-002. |
| `seed_examples_reciprocated_to_feature_patterns_arrays` | scenario | `[file, graph]` | FT-066.patterns, FT-068.patterns, FT-069.patterns include the seeds that listed them in `examples:`. |
| `seed_catalog_dogfoods_every_adr_050_field` | invariant | `[graph]` | Every front-matter field defined in ADR-050 is exercised by at least one seed (schema-completeness check). |
| `seed_pat_003_body_demonstrates_observe_assertion_shape` | scenario | `[file]` | PAT-003's body contains a code snippet showing the file-observation assertion shape (regex check for the assertion pattern from TC-778). |
| `ft_075_exit_criteria_seed_pattern_catalog` | exit-criteria | n/a | Aggregator; cargo gates green; all three seeds present on disk and surfaced in `product pattern list`. |
