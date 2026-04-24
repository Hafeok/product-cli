---
id: FT-055
title: Feature Functional Specification Section
phase: 5
status: planned
depends-on:
- FT-003
- FT-011
- FT-018
adrs:
- ADR-002
- ADR-006
- ADR-013
- ADR-047
tests:
- TC-681
- TC-682
- TC-683
- TC-684
- TC-685
- TC-686
- TC-687
- TC-688
- TC-689
- TC-690
- TC-691
- TC-692
- TC-693
- TC-694
- TC-695
- TC-696
- TC-697
domains:
- api
- data-model
domains-acknowledged:
  ADR-040: W030 surfaces through the existing `product graph check` pipeline and does not alter `product verify` stages. No new verify stage or LLM-boundary hook; the completeness check is structural graph validation, not test execution. The feature is orthogonal to the unified verify pipeline.
  ADR-045: W030 does not interact with planning annotations. Due dates and started tags remain advisory; the functional-spec completeness check is independent of the planning surface. Features with or without a `due-date` are checked identically.
  ADR-018: Test coverage for W030 follows ADR-018 Design 2 (session-based integration tests) — session tests ST-340 through ST-355 are tracked as TC-681 through TC-696 and are session-style graph-check fixtures, consistent with ADR-018's cross-cutting test-strategy obligation.
  ADR-046: W030 is a structural completeness warning; it neither emits nor consumes cycle-time data. The feature adds a graph-check rule, not a planning or forecasting surface.
  ADR-044: W030 surfaces via `product graph check` and the feature-status transition gate — neither interacts with the interactive request builder draft lifecycle. The hint text directs users to `product request change`, which works identically through either the builder or a hand-written YAML (ADR-044's identical-semantics invariant is preserved).
  ADR-041: No absence TCs or ADR removes/deprecates interaction — W030 is a new structural warning that augments the existing graph-check surface. Nothing is removed or deprecated by this feature; TC types used are scenario and exit-criteria only.
  ADR-042: Uses only existing TC types — `scenario` for the sixteen behavioural W030 tests and `exit-criteria` for the consolidated check-list. No new structural or custom TC types are introduced; ADR-042's reserved-structural / open-descriptive partition is unchanged.
  ADR-043: 'Implementation follows the slice + adapter pattern: the pure `src/feature/body_sections.rs` parser is a new slice with no I/O, and W030 emission is added to the existing `src/graph/validation.rs` check pipeline — no monolithic handler introduced. Config plumbing stays in `src/config.rs` alongside the existing FeaturesConfig peers.'
  ADR-038: 'W030 does not introduce new request shapes. The `set: body` mutation that fixes a missing section is the existing body-update operation already supported by the request model. No request-schema changes; W030 uses the established `product request change` hint surface.'
---

## Description

A feature's markdown body gains a fixed-structure `## Functional Specification`
section with seven mandatory subsections — Inputs, Outputs, State, Behaviour,
Invariants, Error handling, Boundaries — plus a sibling `## Out of scope` H2.
Together they capture the behavioural contract an LLM needs to implement the
feature without guessing. `product graph check` parses feature bodies and emits
`warning[W030]` when a required section is absent. Promotion to error tier is
opt-in via `[features].completeness-severity = "error"`.

The design lives in ADR-047 (cross-cutting). The full reference document is
[`docs/product-functional-spec.md`](docs/product-functional-spec.md).

---

## Depends on

- **FT-003** — feature front-matter schema; W030 parses the body portion of the
  markdown file, but section presence is checked against config from the same
  `product.toml` that declares schema-version.
- **FT-011** — context bundle format; the spec is part of the feature body and
  inherits the existing context-bundle assembly — no new assembly code.
- **FT-018** — validation and graph health; W030 is added to the same
  `graph check` channel as the existing W001–W023 warnings.

---

## Functional Specification

### Inputs

- **Feature markdown bodies** under `[paths].features` (default
  `docs/features/`). Parser input is the post-front-matter body string
  (everything after the closing `---`), already available on `Feature.body`
  after `parser::parse_feature`.
- **`[features]` section** in `product.toml`:
  - `required-sections: [String]` (default `["Description",
    "Functional Specification", "Out of scope"]`) — top-level H2 headings
    every non-stub feature body must contain.
  - `functional-spec-subsections: [String]` (default `["Inputs", "Outputs",
    "State", "Behaviour", "Invariants", "Error handling", "Boundaries"]`) —
    H3 headings required under `## Functional Specification`.
  - `required-from-phase: Integer` (default `1`) — features with
    `phase < required-from-phase` are exempt from W030. Allows stub
    features from early migration to stay W030-clean.
  - `completeness-severity: Enum{"warning"|"error"}` (default `"warning"`) —
    when `"error"`, W030 becomes E-class and blocks `planned → in-progress`
    transitions in `product feature status`.

### Outputs

- **On missing section (severity = warning):**
  - `product graph check` writes a W030 diagnostic to stderr using the
    rustc-style format (ADR-013). Exit code 2 when no errors are present.
  - JSON form (`--format json`): a warning entry in the `warnings` array
    with `code: "W030"`, `file: <feature path>`, `detail`, and the list of
    missing section names in `detail`.
- **On missing section (severity = error):**
  - W030 is promoted to E-class; `product graph check` exits 1.
  - `product feature status FT-NNN in-progress` refuses the transition with
    an error message that lists missing sections.
- **On all sections present:** no W030 output. Other checks continue unchanged.
- **Schema surface:** `product schema feature` output documents the
  feature-body structure convention in addition to the YAML front-matter
  schema (no wire change — additive markdown in the rendered output).

### State

Stateless. W030 is computed from the parsed feature body and the
`[features]` config in a single pass; no data is retained between
invocations. Section detection results are not cached.

### Behaviour

Per `product graph check` invocation (after feature parsing):

1. Load `[features]` config from `product.toml`. Apply defaults for any
   unset keys.
2. For each parsed feature `f` in `graph.features`:
   1. If `f.front.phase < required_from_phase`, skip — stub feature exempt.
   2. Parse `f.body` into H2 and H3 section headings with a deterministic
      markdown-aware parser (not a line-regex — ATX-style `## Title` and
      `### Title` headings only, excluding fenced code blocks).
   3. Compute `missing_top = required_sections \ present_h2`.
   4. If `"Functional Specification"` is in `present_h2`, compute
      `missing_sub = functional_spec_subsections \ present_h3_under_fs`.
      Otherwise `missing_sub = ∅` (the parent section is already flagged
      in `missing_top`; do not double-report every subsection).
   5. If `missing_top ∪ missing_sub` is non-empty:
      - When `completeness_severity = "warning"`: push a `W030` warning
        diagnostic listing every missing section.
      - When `completeness_severity = "error"`: push as an error-tier
        diagnostic with the same code `W030` (tier elevated; code stable).
3. Section presence uses *exact heading text* match after whitespace
   trimming. Case-sensitive: `### behaviour` does not match
   `### Behaviour`. This matches the documented contract in
   `docs/product-functional-spec.md` and keeps the check deterministic.
4. A section is considered *present* if its heading line exists and the
   section body contains at least one non-whitespace character before the
   next heading of the same or higher level. An empty-meaning entry such
   as `Stateless. No data is retained between requests.` satisfies this.
5. Fenced code blocks (```` ``` ```` delimited) are not scanned — headings
   inside code samples do not count as real sections.

Per `product feature status` transition to `in-progress`:

1. Same W030 computation as above for the target feature only.
2. If `completeness_severity = "error"` and any W030 findings exist:
   refuse the transition with exit code 1 and print the findings.
3. Otherwise: proceed with the normal transition.

### Invariants

- **Heading parsing is deterministic.** Given the same body string and
  config, the parser returns the same `(present_h2, present_h3)` set on
  every invocation. No non-determinism from hash iteration or parallel
  work.
- **Code fences suppress headings.** A line starting with `## ` or
  `### ` inside a fenced code block is never counted as a section. This
  keeps markdown examples in feature bodies from producing false
  positives.
- **Config absence preserves defaults.** A `product.toml` with no
  `[features]` section still applies the documented defaults. Omitting
  the section is not a repo error; overriding keys is opt-in.
- **W030 code number is stable.** The warning code `W030` does not
  change when severity is promoted to error — only the tier changes.
  Callers filtering CI output on the code do not break.
- **Empty-meaning sections are distinguishable from absent sections.**
  A `### State` followed by `Stateless. No data is retained between
  requests.` satisfies W030; a feature with no `### State` heading does
  not.

### Error handling

- **Malformed markdown body (unterminated fence, etc.).** The parser
  falls back to best-effort heading extraction; unterminated fences are
  treated as "everything after the fence is code". W030 still runs and
  reports whatever headings were found. No parse error is raised — the
  front-matter layer already rejects front-matter-level malformations.
- **Config parse error in `[features]`.** Surfaced as a normal
  `config::ProductConfig::load` error before `graph check` runs. The
  per-feature W030 pass never executes with invalid config.
- **Unknown config key in `[features]`** (e.g. a typo like
  `required-from-pahse`). Handled by serde's `#[serde(deny_unknown_fields)]`
  posture if present, otherwise silently ignored; the existing config
  parser posture is retained. No new error code introduced.
- **Large feature bodies.** The parser is O(body length) — a single
  pass over the lines. No pathological input is reported from the
  existing feature bodies at the current scale; no defensive size cap
  is added.

### Boundaries

- **Heading with trailing whitespace or trailing punctuation.**
  `## Functional Specification  ` (trailing spaces) matches;
  `## Functional Specification:` (trailing colon) does not — the match
  is on the trimmed heading text exactly. This matches the documented
  convention in `docs/product-functional-spec.md`.
- **Duplicate H3 headings under `## Functional Specification`.**
  Permitted — a feature may have two `### Boundaries` blocks and still
  satisfy the check. Duplicates are deduplicated when computing
  `present_h3_under_fs`.
- **H4+ headings inside a required subsection.** Ignored for W030
  purposes — deeper subdivision is allowed and does not affect section
  presence.
- **Headings before the Description section.** Permitted; section order
  is not enforced by W030. Only section presence is checked. Callers
  who want to enforce order can use `product gap check` with a custom
  rule (out of scope for this feature).
- **Feature body consisting of only the front-matter closing `---`
  followed by a single newline (no content).** All required sections
  absent; W030 fires listing each required top-level section.
- **Feature with `phase = 0`.** Skipped (stub) with the default
  `required-from-phase = 1`.
- **Feature promoted from phase 0 to phase 1 without adding sections.**
  W030 fires on the next `graph check` — the exemption is by current
  phase, not historical phase.
- **"Out of scope" appearing as an H3 under Functional Specification
  instead of an H2 sibling.** Does not satisfy the top-level check; W030
  reports `Out of scope` as missing at the H2 level. The documented
  contract is specifically that `## Out of scope` is a sibling of
  `## Functional Specification`, not nested.
- **Section present with only whitespace content.** Treated as absent.
  The content-presence check requires ≥ 1 non-whitespace character
  between the heading and the next heading of equal or higher level.

---

## Out of scope

- **Semantic validation of section content.** W030 is a *structural*
  check — it verifies section presence, not that the Boundaries section
  correctly enumerates every edge case. Content-quality review belongs
  in `product gap check` (G-codes) or in human review, not in W030.
- **Enforcing section order.** W030 does not require `### Inputs` to
  precede `### Outputs`. Order is a convention documented in
  `docs/product-functional-spec.md`; callers who want order enforcement
  can write a separate rule.
- **Auto-generating missing sections.** No LLM prompt is fired when
  W030 detects a missing section. The hint directs the user to
  `product request change` with a `set: body` op; the generation is a
  separate workflow (`product context FT-NNN | claude "write a Functional
  Specification for this feature"`).
- **A separate `SPEC-XXX` artifact type.** Rejected in ADR-047. The
  spec lives in the feature body; no new artifact type, no new graph
  edge, no new MCP tool.
- **Retroactive completeness for abandoned features.** Features with
  `status: abandoned` are not checked — the spec has no purpose once
  the feature is abandoned. Matches the pattern used by W001/W002 for
  abandoned features (see `graph::validation::check_orphaned_tests`).
- **Enforcement via `product verify`.** W030 runs in `product graph
  check` only, not as an extra verify stage. `product verify` remains
  focused on TC execution (ADR-021, ADR-040); spec completeness is a
  graph-check concern, not a test-execution concern.
- **Per-feature override of `completeness-severity`.** The severity is
  repo-wide in `product.toml`. Features cannot opt out individually —
  that would undermine the "every feature has a spec" invariant teams
  opt into by setting severity to `error`.

---

## Implementation notes

- **`src/config.rs`** — new `FeaturesConfig` struct with the four keys
  above, wired into `ProductConfig` as an optional `[features]` section
  with a `Default` impl that matches the documented defaults.
  `completeness-severity` parses to an enum (`Warning` | `Error`) with a
  custom `Deserialize` that accepts the string forms.
- **`src/feature/body_sections.rs`** — new pure module. Exposes
  `parse_body_sections(body: &str) -> BodySections` returning
  `{ h2: Vec<String>, h3_under: HashMap<String, Vec<String>>, h2_empty:
  HashSet<String>, h3_empty: HashMap<String, HashSet<String>> }`. Handles
  ATX-style headings, skips fenced code blocks, and tracks whether each
  section has any non-whitespace content before the next
  same-or-higher-level heading. Fully unit-tested in
  `body_sections_tests.rs`.
- **`src/graph/functional_spec_validation.rs`** — new module. Exposes
  `check_functional_spec(graph: &KnowledgeGraph, config:
  &FeaturesConfig) -> (Vec<Diagnostic>, Vec<Diagnostic>)` returning
  `(errors, warnings)` — warnings by default, errors when severity is
  promoted. Called from `graph::validation::check_with_config`.
- **`src/graph/validation.rs`** — wire the new check into
  `check_with_config` after the existing checks. Feature-gate on
  `config.is_some()` because the check consumes `FeaturesConfig`.
- **`src/feature/status_change.rs`** — in the `planned → in-progress`
  transition, call the functional-spec check for the single target
  feature. If `completeness_severity = "error"` and any findings exist,
  refuse the transition (new error variant surfacing the missing
  sections).
- **`product.toml`** — add the `[features]` section with the default
  values explicitly (future-proofs against default changes).
- **Unit tests** cover: ATX heading detection, fenced code skipping,
  empty-meaning vs absent distinction, duplicate-subsection handling,
  `required-from-phase` exemption, severity promotion, and the
  status-transition gate.
- **Integration tests** exercise `product graph check` against a fixture
  repo with a mix of complete and incomplete feature bodies. JSON output
  shape is locked by a dedicated TC.
- **Documentation** — `docs/product-functional-spec.md` already exists
  as the reference. This feature makes it executable.

---

## Acceptance criteria

A developer running on a populated test fixture can:

1. Run `product graph check` on a repo with a feature missing
   `### Behaviour` and observe the W030 warning with the missing
   section listed.
2. Run `product graph check --format json` and parse the output —
   `warnings[]` contains a W030 entry with `file`, `detail` listing the
   missing sections, and the stable code string `"W030"`.
3. Set `[features].completeness-severity = "error"` in `product.toml`
   and re-run the same `graph check` — same feature now surfaces W030
   in the `errors` array and `graph check` exits 1.
4. Attempt `product feature status FT-NNN in-progress` on a feature
   missing required sections while severity is `"error"` — the
   transition is refused with a listing of the missing sections.
5. Add a `### State\n\nStateless. No data is retained between
   requests.\n` section to a feature lacking State and re-run — W030
   clears for that feature.
6. Point `[features].required-from-phase = 2` at a phase-1 stub feature
   with no required sections — W030 does not fire for that feature.
7. Use a code-fenced markdown example inside a feature body containing
   `## Functional Specification` text — the parser does not count the
   fenced instance; the real section is still required.
8. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` and observe all pass.

See the exit-criteria TC for the consolidated check-list.

---

## Follow-on work

- **Content-quality G-codes.** Once W030 ships and teams are used to
  the structural check, a `product gap check` rule can fire G-codes for
  sections present but shallow (e.g. `### Boundaries` containing one
  bullet when three are probably warranted). Out of scope here.
- **LLM-assisted section drafting.** A `product feature draft-spec
  FT-NNN` command could spawn `claude -p` with the feature context
  bundle and ask it to produce a first draft of the missing sections.
  Pattern already exists for `scripts/generate-docs.sh`; making it a
  first-class command is a separate feature.
- **Spec-to-TC cross-references.** Convention in the doc allows TCs
  to reference the subsection they validate ("Verifies Functional
  Specification / Behaviour step 7"). A future gap-check rule could
  verify that every numbered behaviour step has at least one TC
  referencing it by step number. Deferred.
