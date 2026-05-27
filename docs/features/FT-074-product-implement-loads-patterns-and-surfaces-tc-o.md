---
id: FT-074
title: product implement Loads Patterns and Surfaces TC observes in the Executor Bundle
phase: 5
status: planned
depends-on:
- FT-070
- FT-071
- FT-072
- FT-058
- FT-068
- FT-063
adrs:
- ADR-050
- ADR-051
- ADR-006
- ADR-021
- ADR-022
- ADR-049
- ADR-043
- ADR-018
- ADR-041
- ADR-040
- ADR-042
- ADR-047
- ADR-048
tests:
- TC-847
- TC-848
- TC-849
- TC-850
- TC-851
- TC-852
- TC-853
domains:
- api
- observability
- testing
domains-acknowledged:
  observability: This feature surfaces TC observes:-declared surfaces into the implement bundle. The observability domain is declared because the bundle now exposes assertion surface metadata, but no new observability decisions are made — surfaces are consumed from FT-072 / ADR-051 rather than originated.
---

## Description

Wire patterns and TC `observes:` surfaces into the `product
implement` pipeline so the executor agent sees both before it
writes a line of code. FT-070..FT-073 give the graph the new
artifacts; FT-072 gives TCs the new field. FT-074 makes them
visible at implement time.

The pipeline change is small and additive. Step 1 of the
implement loop (per FT-058 / ADR-021) currently assembles a
context bundle and hands it to the agent. After FT-074, the
bundle:

1. Includes every pattern in `feature.patterns:` in topo order
   following `requires:` edges (FT-071 already produces this for
   `product context`; FT-074 wires it into the implement bundle).
2. Surfaces each linked TC's `observes:` list adjacent to its
   body, so the executor's assertion shape is visible at glance.
3. Includes a hard-constraint reminder block citing ADR-051: "Tests
   under this feature must assert against the named surfaces, not
   on the response envelope alone."

The objective is that the executor agent cannot drift back into
envelope-only stubs because the constraint is in front of it at
every turn. Combined with the FT-072 gate (which blocks merge),
this closes the loop end-to-end.

---

## Depends on

- **FT-070** — Pattern Artifact. The bundle needs the parsed
  patterns to render them.
- **FT-071** — Pattern in Graph Algorithms. Topo-sort over
  `requires:` lives here.
- **FT-072** — TC Observability. The `observes:` field is what
  the bundle surfaces.
- **FT-058** — Enforce TC Runner Configuration. The implement
  pipeline owns the Step 0 / Step 0a chain this feature extends.
- **FT-068** — Convention-Derived TC Runner Config. The runner
  auto-fill Step 0a runs before the bundle is built; FT-074
  adds a Step 1a (or extends Step 1) but does not move other
  steps.
- **FT-063** — Per-Model Context Bundle Templates. The bundle
  template needs to render the new sections.

---

## Functional Specification

### Inputs

- `product implement FT-XXX` — existing command; behaviour
  augmented as described.
- Feature front-matter `patterns: [PAT-A, PAT-B]` — read from
  disk via the existing graph load.
- TC front-matter `observes: [file, graph]` — read identically.
- The configured implement prompt template (FT-056
  `[implement].prompt-path` override, or the default
  `docs/prompts/implement-v1.md`).

### Outputs

- The context bundle passed to the executor agent now contains:
  1. The feature body (unchanged).
  2. A "## Patterns" section listing each pattern in topo order
     with its full body (already produced by FT-071 for
     `product context`).
  3. A "## Test Criteria" section that for each TC includes:
     - The TC body (existing).
     - A line `observes: [<surfaces>]` rendered immediately
       before the TC body.
     - When `observes:` is empty for a required-type TC, an
       inline `# WARNING: TC missing observes per ADR-051` comment
       (this is also caught by FT-072's graph-check gate; the
       inline note is belt-and-braces).
  4. A "## Hard constraints" block (the existing block from
     FT-068's pipeline) extended with the ADR-051 reminder line.
- The implement prompt template renders the new sections via
  template variables (`{{patterns}}`, `{{tc_observes_table}}`,
  `{{hard_constraints}}`). Existing templates that omit these
  variables continue to work — sections simply do not appear.

### State

- Extension to `src/implement/pipeline.rs`:
  - The bundle-build step calls the shared
    `context::build_bundle` (which FT-071 already extends with
    patterns).
  - A new helper `src/implement/observes_table.rs` produces the
    TC `observes:` surface table for inclusion in the prompt.
- Extension to `docs/prompts/implement-v1.md`:
  - New section under "Context" naming the patterns block.
  - Hard-constraint line referencing ADR-051.
- Update to per-model templates (FT-063):
  - The default template includes the new variables; the existing
    `claude-opus`, `gpt-4-markdown`, and `human` templates gain
    the patterns and observes sections.

### Behaviour

1. **Pipeline.** `product implement FT-XXX` steps:
   - Step 0a (FT-068): runner auto-fill.
   - Step 0 (FT-058): preflight (includes the FT-072 gate
     once that lands — TC missing `observes:` blocks here).
   - Step 1: assemble context bundle. After FT-074, the bundle
     includes patterns and the TC observes surfaces.
   - Steps 2+: existing agent invocation and verify.

2. **Bundle assembly.** `build_bundle(graph, feature_id,
   options)` (the shared function FT-071 extends) returns a
   `Bundle` struct with patterns ordered by topo over `requires:`.
   FT-074 adds a `tc_observes: Vec<(String, Vec<String>)>` map
   to the bundle, and the template renders the table.

3. **Hard constraint block.** The implement prompt template's
   "Hard constraints" section gains a line:
   > Tests must assert against the surface(s) declared in each
   > TC's `observes:` field. A test that asserts only on a
   > response envelope when the operation has a disk side-effect
   > violates ADR-051 and will be rejected.

4. **Per-model template parity.** The three existing templates
   under `docs/context-templates/` are updated to expose
   `{{patterns}}` and `{{tc_observes_table}}`. Templates without
   these variables produce a bundle missing the data — the
   pipeline does not fail, but the executor loses the signal.
   A new test verifies the default template renders both
   sections.

5. **MCP parity.** `product implement` is a CLI-only pipeline;
   no MCP exposure. The bundle assembly is reusable from MCP via
   `product_context` (which already inherits FT-071's patterns
   work) — F5 does not duplicate that surface.

### Invariants

- For every `product implement FT-XXX` invocation, the bundle
  passed to the executor contains every PAT in `feature.patterns:`
  expanded transitively over `requires:`.
- The bundle's pattern section is rendered in topo order: every
  prerequisite PAT appears before its dependant PATs.
- The bundle's TC section renders each TC's `observes:` list
  inline with the TC body.
- The bundle's "Hard constraints" block contains the ADR-051
  reminder line.
- Templates lacking the new variables continue to render a
  legacy bundle without failing the pipeline (backwards-compat).
- The default template renders all three new sections present
  (regression guard).

### Error handling

- **Empty `feature.patterns:`** — the "## Patterns" section is
  omitted entirely. Not an error; consistent with how the bundle
  treats features with no linked ADRs today.
- **Missing pattern referenced in `feature.patterns:`** — broken
  link reported as E002 by the existing parser; the bundle
  assembly fails fast with the same code.
- **TC missing `observes:`** — for required-for types this is
  blocked by FT-072's gate at Step 0; if the pipeline somehow
  reaches Step 1 with such a TC (e.g. when running with
  `[tc-observability].required-from-phase` set high), the
  bundle inlines the warning comment described above.
- **Template lacks `{{patterns}}`** — bundle renders without the
  section; no error.

### Boundaries

- **In scope:** the pipeline bundle extension; the implement
  prompt template update; the per-model template updates; the
  observes-table helper.
- **Out of scope:** any change to the agent-invocation step or
  the verify step (FT-058 / FT-068 own those).
- **Out of scope:** planner/executor model split (the brief's
  noted future direction — patterns + observability make it
  possible; they do not require it).
- **Out of scope:** changes to `product context` beyond what
  FT-071 already provides — F5 reuses `build_bundle` rather
  than diverging.
- **Out of scope:** new MCP tools (`product implement` is
  CLI-only).

---

## Out of scope

- Planner/executor model-size split.
- Changes to other commands' bundle assembly.
- New MCP tools.
- Mutation testing or other deferred backstops referenced by
  ADR-051's rationale.

---

## Implementation notes

- **`src/implement/observes_table.rs`** — pure helper that takes
  a slice of `(TcId, TcFrontMatter)` and emits a markdown
  fragment listing each TC's `observes:` surfaces. ~60 lines
  including tests.
- **`src/implement/pipeline.rs`** — small edit: route the
  bundle through `context::build_bundle` (instead of the
  current inlined assembly, if any), then pass `bundle.patterns`
  and the new observes table through the prompt template's
  `format!`. ~30 LoC delta.
- **`docs/prompts/implement-v1.md`** — add new "## Patterns"
  reference and the hard-constraint line. Bump version (the
  prompts registry uses versioned filenames per FT-022).
- **`docs/context-templates/*.tera`** (or whatever the per-model
  template format is) — add the `{{patterns}}` and
  `{{tc_observes_table}}` blocks to each of the three shipped
  templates. The default template is the regression-guarded
  one.
- **AGENTS.md** — the "Implementation Workflow" section gains
  bullet referencing the patterns + observability inclusion.
- **File-length budget:** new file ≤ 100 lines; pipeline.rs
  delta keeps the file under the 400-line cap.
- **Concurrency:** read-side only at bundle assembly; no new
  locking.

---

## Acceptance criteria

A developer can:

1. Author a feature citing `patterns: [PAT-A]` (where PAT-A
   requires PAT-B), run `product implement FT-XXX --dry-run`,
   and observe the rendered bundle includes both PAT-A and
   PAT-B in topo order under a "## Patterns" section.
2. Author the same feature with linked TCs declaring `observes:
   [file, graph]`, run `product implement FT-XXX --dry-run`,
   and observe each TC's `observes:` list rendered inline with
   its body.
3. Read the bundle's "Hard constraints" block and confirm the
   ADR-051 reminder line is present verbatim.
4. Run `product implement FT-XXX` against a model whose template
   omits `{{patterns}}` — the pipeline completes (legacy
   templates still work); the agent simply doesn't see the
   section.
5. Run the same against the default template — the bundle
   contains all three new sections (pattern section, observes
   table, hard-constraint line).
6. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

---

## TC scaffolding plan

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `implement_bundle_includes_patterns_in_topo_order` | scenario | `[stdout]` | `product implement --dry-run` produces a bundle with patterns ordered per `requires:`. |
| `implement_bundle_renders_tc_observes_inline_with_tc_body` | scenario | `[stdout]` | Each TC's `observes:` list appears adjacent to its body in the bundle. |
| `implement_bundle_contains_adr_051_hard_constraint_line` | scenario | `[stdout]` | The "Hard constraints" block of the bundle contains the ADR-051 reminder verbatim. |
| `implement_pipeline_works_with_template_lacking_new_variables` | scenario | `[stdout, exit-code]` | A template missing `{{patterns}}` still runs (legacy compat). |
| `implement_default_template_renders_all_new_sections` | scenario | `[stdout]` | The default template's rendered bundle contains every new section (regression guard). |
| `implement_skips_pattern_section_when_feature_has_none` | scenario | `[stdout]` | A feature with `patterns: []` produces a bundle without the patterns section (no empty header). |
| `ft_074_exit_criteria_implement_patterns_and_observes` | exit-criteria | n/a | Aggregator; cargo gates green; the implement prompt's version is bumped. |
