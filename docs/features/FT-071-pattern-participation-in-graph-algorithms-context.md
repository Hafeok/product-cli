---
id: FT-071
title: Pattern Participation in Graph Algorithms — Context, Impact, Centrality, Validation
phase: 5
status: complete
depends-on:
- FT-070
- FT-006
- FT-011
- FT-016
- FT-018
- FT-024
- FT-069
adrs:
- ADR-050
- ADR-012
- ADR-006
- ADR-020
- ADR-013
- ADR-043
- ADR-018
- ADR-049
- ADR-048
- ADR-042
- ADR-040
- ADR-047
- ADR-051
- ADR-041
tests:
- TC-820
- TC-821
- TC-822
- TC-823
- TC-824
- TC-825
- TC-826
- TC-827
- TC-828
- TC-829
domains:
- api
- data-model
- observability
domains-acknowledged:
  observability: This feature surfaces patterns through existing observability affordances (graph check findings, impact tree, centrality). It declares the observability domain because it adds new graph-check diagnostics, but it does not establish new observability decisions — those would belong in a separate ADR. The existing observability ADRs (ADR-036, ADR-039) govern tag-based tracking and log hash-chains, which this feature does not modify.
---

## Description

Wire patterns (introduced by FT-070 per ADR-050) into the graph
algorithms that already power features and ADRs: context bundle
assembly, impact analysis, betweenness centrality, and `product
graph check` validation rules. FT-070 added the artifact type, the
parser, and the CRUD surface; FT-071 makes patterns observable
through every read-side affordance the graph offers.

The premise of ADR-050 is that patterns are first-class nodes — not
documentation strings tucked into another artifact. A first-class
node participates in walks (context), reverse walks (impact),
centrality (architectural-importance signal), and structural health
(graph check). Without F2, patterns are merely files; with F2,
patterns are graph citizens.

---

## Depends on

- **FT-070** — Pattern Artifact (parse, schema, CRUD). The schema
  and parser this feature builds on.
- **FT-006** — Impact Analysis. Owns the reverse-edge traversal that
  this feature extends to patterns.
- **FT-011** — Context Bundle Format. Owns the bundle assembly path
  that this feature extends.
- **FT-016** — Graph Model. Owns the Brandes centrality
  implementation patterns participate in.
- **FT-018** — Validation and Graph Health. Owns the `product graph
  check` framework where new diagnostics land.
- **FT-024** — Graph Intelligence. Owns the centrality command
  surface where patterns become reportable.

---

## Functional Specification

### Inputs

- `product context FT-XXX --depth N` — existing flag set; no schema
  change.
- `product impact PAT-XXX` — new id prefix accepted by the impact
  command.
- `product impact ADR-XXX` and `product impact FT-XXX` — existing
  commands; both extended to report PATs in their dependency
  surfaces.
- `product graph central` — existing command; output extended with a
  pattern axis (or a `--include patterns` flag, see Behaviour).
- `product graph check` — existing command; new diagnostic codes
  introduced (see Error handling).
- MCP parity for the above: `product_impact`, `product_context`,
  `product_graph_central`, `product_graph_check` — all
  read-side tools — extended to expose the same data.

### Outputs

- **Context bundle.** Bundles for a feature that has `patterns:
  [PAT-A, PAT-B]` include both PATs in their body, ordered such that
  every prerequisite appears before its dependant (topo-sorted over
  `requires:` edges). The bundle layout is feature →
  patterns (topo order) → ADRs → TCs → dependencies, with the
  feature → patterns section labelled "## Patterns" in the rendered
  template. The bundle metadata (`bundle:` block in feature
  front-matter, FT-040) gains a `patterns: <count>` field.
- **Impact (`product impact PAT-XXX`).** Lists every feature whose
  `patterns:` array contains the PAT id; every pattern whose
  `requires:` array contains it; every ADR cited in the PAT's
  `adrs:` (downstream not upstream — the ADR drives the pattern).
  The output reuses the existing impact-tree renderer.
- **Centrality.** `product graph central --include patterns` (new
  flag, default off for backwards-compat with existing tooling)
  computes betweenness over a graph that now includes PAT nodes.
  When the flag is omitted, the algorithm proceeds as today and
  PATs are excluded entirely (no participation, no signal noise).
- **Graph check.** New findings emitted for: requires-cycle (error,
  new code E0XX); deprecated-pattern cited by live feature
  (warning, new code W0XX); pattern body missing required section
  (warning, new code W0XX — the parser hook from FT-070 finally has
  a wired diagnostic).

### State

- No new persistent state. All additions are computations over the
  graph FT-070 already produces.
- New module `src/graph/pattern_topo.rs` exposes
  `topo_sort_patterns(graph: &KnowledgeGraph, ids: &[String]) ->
  Result<Vec<String>, ProductError>`. Pure function; returns the
  input ids in topo order following `requires:` edges, or
  `RequiresCycle` on detection.
- Extension to `src/context/bundle.rs` adds a `Patterns` section to
  `BundleSections`; pure render function emits the section text.
- Extension to `src/impact/` adds a `PatternImpact` plan struct;
  pure traversal mirrors `feature_impact_plan` / `adr_impact_plan`.
- Extension to `src/graph/centrality.rs` accepts an `include_kinds:
  &[ArtifactKind]` parameter; existing call sites pass the legacy
  set (FT, ADR, TC, DEP); the new `--include patterns` path passes
  the augmented set.
- Extension to `src/graph/check.rs` registers three new
  diagnostics (codes assigned at implementation time; tentatively
  W031 already taken by FT-070's body-section hook, so the new
  ones are E0XX / W0XX adjacent — implementation allocates the
  next free codes).

### Behaviour

1. **Context bundle assembly.** When assembling a bundle for
   `FT-XXX`, the bundle pipeline:
   a. Reads `FT-XXX.patterns` from the feature front-matter.
   b. For depth ≥ 1, transitively walks `pattern.requires` edges
      to include every prerequisite pattern (depth-bounded by the
      `--depth` flag).
   c. Topo-sorts the collected pattern ids over the `requires:`
      DAG.
   d. Renders each pattern as a "## PAT-NNN: <title>" subsection
      under a top-level "## Patterns" heading.
   e. Tracks the count under `bundle.patterns` in front-matter
      when `product context --measure` runs.

2. **Impact traversal.** `product impact PAT-XXX` walks:
   - All features whose `patterns` array contains the id
     (forward edge).
   - All patterns whose `requires` array contains the id (forward
     edge in the requires DAG).
   - All ADRs in the PAT's `adrs` array (downstream edge — the
     ADR is what the pattern operationalises; changes to the
     pattern do not propagate to the ADR, but the relationship is
     surfaced for human navigation).
   `product impact ADR-XXX` is extended to additionally list any
   pattern that has the ADR in its `adrs` array.
   `product impact FT-XXX` is extended to additionally list every
   pattern in the feature's `patterns` array.

3. **Centrality.** `product graph central` accepts a new
   `--include patterns` flag. When set, the algorithm builds the
   adjacency map over the union of (FT, ADR, TC, DEP, PAT) nodes
   and the union of their link arrays (`patterns:`, `requires:`,
   `examples:`, plus the existing edges). The reported centrality
   ranking includes PAT ids in the result list. Without the flag,
   behaviour is unchanged from today (backwards-compatibility for
   existing scripts and the AGENTS.md "top-5 ADRs" workflow).

4. **Graph check — `requires:` cycle.** During the structural pass,
   `check_with_config` invokes `topo_sort_patterns` over every PAT.
   On `RequiresCycle`, emit a new error finding listing the cycle.
   The diagnostic carries the cycle as a path
   (`PAT-A → PAT-B → PAT-A`).

5. **Graph check — deprecated pattern cited by live feature.** For
   every PAT whose status is `deprecated`, walk the reverse edge to
   features. Any feature with `status ∈ {planned, in-progress}`
   citing the deprecated PAT emits a warning. Features with
   `status = abandoned` or `complete` are exempt — complete
   features have already shipped against the pattern; abandoned
   ones are no longer active.

6. **Graph check — pattern body missing required section.** FT-070
   added the config keys and the parser hook; this feature wires
   them through `check_with_config`. When
   `[patterns].body-severity = "warning"` (default), emit a warning
   per missing section. When `= "error"`, escalate. The diagnostic
   shape mirrors W030 exactly (heading paths, hint with the YAML
   snippet to add).

7. **MCP parity.** `product_graph_check`, `product_context`,
   `product_impact`, and `product_graph_central` all surface the
   new behaviour over MCP. The parity invariant from FT-069 holds:
   MCP JSON output equals CLI `--format json` output
   byte-for-byte against any fixture.

### Invariants

- For every PAT, `requires` forms a DAG: no node reaches itself.
- For every bundle assembled with `--depth ≥ 1`, every pattern in
  the bundle appears after each of its prerequisites in the
  rendered output (topo invariant).
- For every PAT cited by a live (planned / in-progress) feature,
  the PAT is `status: live` (or a warning is emitted).
- For every PAT, the body contains the configured required H2
  sections (or a warning is emitted; escalates to error when
  configured).
- `product graph central` without `--include patterns` returns
  byte-identical output to the pre-FT-071 implementation.
- `product graph check --format json` output (MCP and CLI) is
  byte-identical for the same fixture.
- `product impact PAT-XXX` lists every artifact that would be
  affected by removing or modifying the PAT — the canonical impact
  semantic from FT-006 generalised to PAT.

### Error handling

- **E0XX** (new, allocated at implementation time as the next free
  E-code) — `requires:` cycle detected. The error names every node
  in the cycle, mirrors E003's rendering (the feature
  `depends-on` cycle code).
- **W0XX** (new) — Deprecated pattern cited by live feature. Names
  the feature, the deprecated PAT, and (if set) the `deprecated-by`
  replacement.
- **W0XX** (new, separate from FT-070's W031 if W031 lands) —
  Pattern body missing required H2 section. Mirrors W030.
- **NotFound** — Existing variant; `product impact PAT-XXX`
  against an unknown id.
- Error code allocation: the implementation picks the next free
  codes in sequence (the existing scheme is sequential by feature;
  this feature lands three new codes adjacent to W031 from FT-070).

### Boundaries

- **In scope:** context bundle integration; impact extension;
  centrality opt-in; three new graph-check diagnostics; MCP parity
  for all four read-side tools.
- **Out of scope:** new write operations on patterns (FT-070
  ships the full CRUD surface; F2 is read-side and validation
  only).
- **Out of scope:** authoring-aware behaviour (`product author
  feature` proposing patterns; `product author pattern`). Those
  land in F4.
- **Out of scope:** `product implement` integration (loading
  patterns into the executor agent's context). That is F5.
- **Out of scope:** the seed catalog (F6). F2 validates that
  patterns work in isolation; F6 dogfoods by populating three of
  them.
- **Out of scope:** semantic similarity / "suggest similar
  patterns" (an LLM-driven feature, not graph-driven).

---

## Out of scope

- All items listed under "Boundaries → Out of scope" above.
- Schema changes to patterns beyond what FT-070 ships.
- New CRUD commands (read-side and validation only).
- Backwards-incompatible changes to `product graph central` — the
  pattern axis is gated behind an opt-in flag.

---

## Implementation notes

- **`src/graph/pattern_topo.rs`** — pure topo-sort with cycle
  detection. ~80 lines including tests. Reuses the depends-on
  cycle detector pattern from `src/feature/depends_on.rs`.
- **`src/context/bundle.rs`** — extend the bundle render path to
  emit a "## Patterns" section between the feature body and the
  ADR list. The ordering is feature → patterns → ADRs → TCs →
  deps, matching how readers traverse from intent to verification.
  Update the existing per-model templates (FT-063) to reference
  the new section; templates that omit `{{patterns}}` continue to
  work (the section simply doesn't appear).
- **`src/impact/`** — add `pattern_impact_plan(graph, id) ->
  ImpactPlan`. The existing CLI adapter `commands::impact`
  dispatches by id prefix; add a `PAT-` arm. Extend the existing
  `feature_impact_plan` and `adr_impact_plan` to include patterns
  in their result sets.
- **`src/graph/centrality.rs`** — extend the function signature to
  accept `include_kinds: ArtifactKindMask`. Default mask preserves
  legacy behaviour. The flag plumbing lives in
  `commands::graph::central`.
- **`src/graph/check.rs`** + `src/graph/full_check.rs` (the
  consolidated check entry point introduced by FT-069) — register
  the three new diagnostics. Each is a small function over the
  graph; aggregation is straightforward.
- **`src/mcp/registry.rs`** — no new tools; the existing
  read-side tools pick up the new behaviour via the shared
  slices.
- **Schema docs** — update `src/commands/schema.rs` and the
  AGENTS.md generator to surface the new `bundle.patterns`
  metadata field.
- **File-length budget:** each new file targets ≤ 200 lines; the
  centrality extension touches the existing centrality module
  without enlarging it materially.
- **Concurrency:** read-side only; no new locking.

---

## Acceptance criteria

A developer can:

1. Author a feature citing `patterns: [PAT-A, PAT-B]` where PAT-B
   requires PAT-A, run `product context FT-XXX --depth 1`, and
   observe PAT-A's body appearing before PAT-B's body in the
   rendered output.
2. Run `product context FT-XXX --depth 1 --measure` against the
   same feature and observe the `bundle.patterns: 2` field
   written to the feature's front-matter.
3. Run `product impact PAT-A` and observe every feature, pattern,
   and ADR linked to PAT-A enumerated in the impact tree.
4. Run `product graph check` against a repo where PAT-A requires
   PAT-B and PAT-B requires PAT-A; observe an error citing the
   cycle.
5. Run `product graph check` against a repo where a `planned`
   feature cites a `deprecated` pattern; observe a warning.
6. Run `product graph check` against a repo where a live pattern
   body lacks one of the configured H2 sections; observe a
   warning (or error, when `[patterns].body-severity = "error"`).
7. Run `product graph central --include patterns` and observe PAT
   ids interleaved with FT / ADR / TC ids in the centrality
   ranking; run without the flag and observe the legacy ranking
   unchanged.
8. Invoke `product_graph_check` over MCP against any of the above
   fixtures and confirm the JSON envelope is byte-identical to
   the CLI `--format json` output (FT-069 parity invariant
   generalised).
9. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

---

## TC scaffolding plan

Every TC scaffolded carries `observes:` per ADR-051, runner config
per CLAUDE.md, and asserts on the named surface.

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `context_bundle_includes_patterns_in_topo_order` | scenario | `[stdout]` | Bundle text contains PAT-A before PAT-B when PAT-B requires PAT-A. |
| `context_bundle_measure_writes_patterns_count` | scenario | `[file]` | `--measure` writes `bundle.patterns: N` to the feature front-matter. |
| `impact_pat_lists_features_patterns_adrs` | scenario | `[stdout]` | `product impact PAT-A` enumerates every linked artifact. |
| `graph_check_requires_cycle_emits_error` | scenario | `[exit-code, stdout]` | A two-PAT cycle causes `graph check` to exit non-zero with the new E-code. |
| `graph_check_deprecated_pattern_cited_by_live_feature_emits_warning` | scenario | `[stdout, exit-code]` | A planned feature citing a deprecated PAT emits the new warning code. |
| `graph_check_pattern_body_missing_section_emits_warning` | scenario | `[stdout]` | A live PAT lacking "Anti-patterns" H2 emits a warning. |
| `graph_central_with_include_patterns_surfaces_pat_ids` | scenario | `[stdout]` | `product graph central --include patterns` returns ids including PAT entries. |
| `graph_central_without_flag_excludes_pats` | scenario | `[stdout]` | `product graph central` (no flag) returns ranking with no PAT ids — backwards-compat. |
| `mcp_graph_check_pattern_findings_match_cli_json` | scenario | `[mcp-response, stdout]` | MCP JSON envelope for `product_graph_check` byte-equals CLI `--format json` for a fixture exercising every new diagnostic. |
| `ft_071_exit_criteria_pattern_graph_integration` | exit-criteria | n/a | Aggregates all the above; cargo gates green. |
