---
id: FT-073
title: Pattern Authoring — author pattern Session and Feature-Aware Pattern Suggestions
phase: 5
status: planned
depends-on:
- FT-070
- FT-071
- FT-022
- FT-021
- FT-066
adrs:
- ADR-050
- ADR-022
- ADR-020
- ADR-043
- ADR-038
- ADR-013
- ADR-018
- ADR-042
- ADR-040
- ADR-049
- ADR-048
- ADR-047
- ADR-041
- ADR-051
tests:
- TC-839
- TC-840
- TC-841
- TC-842
- TC-843
- TC-844
- TC-845
- TC-846
domains:
- api
- data-model
domains-acknowledged: {}
---

## Description

Add the authoring affordances that make patterns first-class in the
human (and agent) workflow. FT-070 + FT-071 give patterns parser
support, CRUD, and read-side graph integration. FT-073 layers
authoring onto that base:

1. **`product author pattern`** — a focused authoring session
   prompt (smaller than `author-feature`) that walks an author
   through scaffolding a PAT against the schema established by
   ADR-050.
2. **`product author feature` extension** — load the current
   pattern catalog into the authoring context and propose matching
   patterns based on the feature's `domains:`. The proposal is
   advisory; the author may accept, reject, or ignore.
3. **MCP authoring tools** — the MCP server gains the
   write-through pattern tools (`product_pattern_new`,
   `product_pattern_status`, `product_pattern_link`,
   `product_pattern_show`, `product_pattern_list`) per the F1
   handler implementations, with the authoring prompt accessible
   via `product_prompts_get`.
4. **`product feature link --pattern PAT-XXX`** — the symmetric
   CLI affordance for the `feature.patterns` field FT-070 added.

The objective is to make pattern citation a normal step in feature
authoring, not a manual chore. The advisory warning if a non-trivial
feature cites no pattern (per the brief's open question) starts as a
soft signal; promotion to error is deferred to a separate decision
once the seed catalog covers enough ground.

---

## Depends on

- **FT-070** — Pattern Artifact (parse, schema, CRUD). The
  authoring surface writes through the slice this feature
  provides.
- **FT-071** — Pattern in Graph Algorithms. The "propose matching
  patterns" path traverses the graph using the algorithms FT-071
  ships.
- **FT-022** — Authoring Sessions. The prompt registration and
  session lifecycle this feature extends.
- **FT-021** — MCP Server. Owns the MCP transport.
- **FT-066** — MCP Parity for Feature/TC Status Writes. Establishes
  the slice-dispatch shape this feature's MCP writes follow.

---

## Functional Specification

### Inputs

- `product author pattern [--title "..."]` — interactive session.
  No required args; the prompt walks the user through everything.
- `product author feature` (existing) — extended to load the
  pattern catalog (status: live) into the session context.
- `product feature link FT-XXX --pattern PAT-YYY` — CLI flag.
- `product_pattern_*` MCP tools — registered by FT-070; the
  authoring prompt makes them composable.
- `product_prompts_list` / `product_prompts_get` — extended to
  include the new `author-pattern-v1.md` prompt.
- `product.toml` — new key `[patterns].suggest-domains` (boolean,
  default `true`) — gates the "propose matching patterns" step in
  `author-feature`.

### Outputs

- `product author pattern` — a typed session that culminates in
  one or more `product_pattern_new` + `product_pattern_link` /
  `product_pattern_status` writes, observed on disk and in the
  request log.
- `product author feature` — when patterns matching the feature's
  domains exist, the session prompts the author with a numbered
  list and accepts a comma-separated selection. The selected PAT
  ids are written into `feature.patterns` via the same
  request-apply batch that creates the feature.
- `product feature link FT-XXX --pattern PAT-YYY` — same writes
  as `product feature link --test` / `--adr`, with bidirectional
  materialisation per ADR-050: feature gets PAT in `patterns:`;
  PAT gets feature in `examples:`.
- New prompt file `docs/prompts/author-pattern-v1.md` shipped
  alongside the existing prompts.

### State

- New session prompt file in `docs/prompts/`.
- New CLI subcommand `product author pattern` registered in
  `commands/author.rs`.
- Extension to `commands/feature_write.rs::feature_link` to
  accept and dispatch the new `--pattern` flag.
- Extension to `commands/author.rs::author_feature` to walk the
  pattern catalog and surface matches.
- New helper module `src/pattern/suggest.rs`:
  - `pub fn suggest_patterns(graph: &KnowledgeGraph, feature_domains:
    &[String]) -> Vec<&PatternArtifact>` — pure, ranks by
    domain overlap then by centrality (using FT-071's pattern-aware
    centrality with `--include patterns`).

### Behaviour

1. **`product author pattern` session lifecycle.**
   a. Loads the system prompt from `docs/prompts/author-pattern-v1.md`
      (registered via `product_prompts_list`).
   b. The prompt instructs the agent to: list existing patterns
      first (no duplicates), examine the relevant ADRs the
      pattern operationalises, propose a draft, scaffold via
      `product_pattern_new`, fill the required body sections,
      link via `product_pattern_link` (adrs, requires,
      examples).
   c. The session closes when the agent calls
      `product_graph_check` and observes no PAT-related warnings
      against the new PAT.
   d. The host process (per FT-022 conventions) refuses to
      auto-commit if `graph check` is dirty on the authored PAT.

2. **`product author feature` extension.**
   a. After the agent has proposed the feature's `domains:` and
      the feature scaffold has been written (mid-session), the
      pipeline runs `suggest_patterns(graph, feature_domains)`.
   b. If matches exist, the prompt context gets a "Matching
      patterns" block listing them (id, title, status, brief
      one-line). The author (or agent) selects which to cite via
      a `product feature link FT-XXX --pattern PAT-YYY` call (or
      via the request batch).
   c. If no matches exist, no prompt block is added — silence.
   d. The `[patterns].suggest-domains = false` flag disables the
      step entirely.

3. **`product feature link --pattern`.**
   a. Adds `PAT-YYY` to `FT-XXX.patterns` (a no-op if already
      present).
   b. In the same atomic batch, adds `FT-XXX` to
      `PAT-YYY.examples` — the FT-070 reciprocation invariant.
   c. Returns the FT-066 `{ writes, reciprocated }` shape.
   d. Validates the PAT exists (NotFound) and is not deprecated
      (warning, not error — the author may intentionally cite a
      deprecated PAT when migrating; the warning fires through
      `graph check`'s deprecated-pattern-cited rule once
      committed).

4. **MCP authoring tools.**
   a. `product_pattern_new`, `product_pattern_status`,
      `product_pattern_link`, `product_pattern_show`,
      `product_pattern_list` are all registered (FT-070 ships
      these as part of CRUD; F4 confirms they are correctly
      exposed and the authoring prompt invokes them).
   b. `product_feature_link` MCP arg gains `pattern: PAT-YYY`
      (symmetric to the CLI flag).
   c. All writes route through the slice (no envelope-only
      stubs — FT-066 invariant).

5. **Pattern-catalog advisory warning (W0XX, soft).** When a
   feature transitions to `status: in-progress` and has zero
   entries in `patterns:`, `product graph check` emits a soft
   warning (new code) suggesting the author review the pattern
   catalog. Default severity `warning`; configurable promotion to
   `error` via `[features].patterns-required-severity` once the
   catalog is mature. Defaults to off entirely until the seed
   catalog lands (F6).

### Invariants

- `product author pattern` always loads existing patterns before
  proposing a new one (no duplicates) — verified by the prompt
  instructions and a session-test that asserts the session reads
  `product_pattern_list` at least once before any
  `product_pattern_new`.
- `product author feature` does not block on the pattern
  suggestion step; an author who declines all suggestions
  progresses normally.
- `product feature link --pattern` produces byte-identical writes
  to the equivalent `product_request_apply` payload (CLI / request
  parity).
- All MCP pattern tools route through the slice and produce
  on-disk writes byte-identical to the CLI (parity invariant from
  FT-066).
- The pattern-suggestion step is gated by configuration and never
  fires when the catalog is empty.

### Error handling

- `NotFound` — `--pattern PAT-999` against an unknown id.
- Deprecation warning — `--pattern PAT-X` where `PAT-X.status =
  deprecated`. The write succeeds; the warning prints to stderr
  and is also raised by `graph check` per FT-071.
- `product author pattern` session failures propagate through the
  existing FT-022 session-error path.
- MCP errors propagate via `format!("{}", e)` per FT-066.

### Boundaries

- **In scope:** the `author-pattern` prompt; the
  `author-feature` extension; the `feature link --pattern` flag;
  the MCP exposure of all pattern tools and the
  `product_feature_link` pattern arg; the suggestion helper; the
  soft advisory warning (off by default until the catalog
  matures).
- **Out of scope:** the seed pattern catalog itself (F6
  dogfoods authoring by populating three patterns).
- **Out of scope:** loading patterns into the `product implement`
  context bundle (F5).
- **Out of scope:** automatic pattern matching by LLM semantic
  similarity — the suggestion is domain-overlap + centrality
  only. Semantic suggestion is a deferred design.
- **Out of scope:** promoting the advisory pattern-citation
  warning to error by default. Per the open question in the brief
  — defer until the catalog covers enough ground.

---

## Out of scope

- All items under "Boundaries → Out of scope" above.
- A `product pattern author` command at the slice level beyond
  what `product author pattern` provides.
- Pattern import / export across repositories (cross-project
  pattern sharing is explicitly speculative per ADR-050).

---

## Implementation notes

- **`docs/prompts/author-pattern-v1.md`** — new file. Mirrors
  the structure of `author-adr-v1.md`: tool-call discipline,
  required body sections, closing graph check. ~80 lines.
- **`src/pattern/suggest.rs`** — pure ranking function. ~100
  lines including tests. Domain overlap (set intersection size)
  is the primary key; pattern centrality (from FT-071) breaks
  ties.
- **`src/commands/author.rs`** — extend `author_feature` to call
  `suggest_patterns` mid-session and inject the matches into the
  prompt context. The injection mechanism reuses the existing
  context-bundle assembly path (the session reads the same bundle
  shape `product context` produces).
- **`src/commands/feature_write.rs::feature_link`** — extend the
  clap arg list with `--pattern`. Dispatch the new arg through
  the existing `feature::plan_link` path (FT-066). `plan_link`
  gains a `pattern: Option<String>` field and emits the
  reciprocal `examples:` write.
- **`src/mcp/registry.rs`** — confirm all pattern MCP tools
  registered by FT-070 work as expected; add the `pattern:`
  field to the `product_feature_link` input schema.
- **`src/graph/check.rs`** — add the new advisory check; gate
  it behind `[features].patterns-required-severity` (default
  `off`, allowed values `off | warning | error`).
- **AGENTS.md** — the "Key MCP Tools" table gains the pattern
  authoring tools; the working protocol mentions calling
  `product_pattern_list` before authoring a new pattern.
- **Prompts registry** — `product_prompts_list` includes
  `author-pattern`; `product_prompts_get` returns its content.
- **File-length budget:** every new file ≤ 200 lines.
- **Concurrency:** writes hold the repo lock; no new primitives.

---

## Acceptance criteria

A developer can:

1. Run `product author pattern` and complete a session that
   results in a new PAT on disk with the required body
   sections, status `live`, and at least one ADR cited.
2. Run `product author feature` for a feature with `domains:
   [api, observability]` against a repo with PAT-A (domains:
   [api]) and PAT-B (domains: [observability]); observe both
   patterns surfaced in the prompt context. Confirm declining
   the suggestion progresses the session.
3. Run `product feature link FT-100 --pattern PAT-001` and
   observe both `FT-100.patterns: [PAT-001]` and
   `PAT-001.examples: [FT-100]` written in the same atomic
   batch (parity with `product_feature_link` test/adr).
4. Invoke `product_feature_link` over MCP with `{ id: "FT-100",
   pattern: "PAT-001" }` and observe the same on-disk result.
5. Invoke `product_pattern_new` over MCP through the authoring
   session and observe a file on disk byte-identical to the CLI
   shape.
6. Run `product graph check` against a repo with FT-100
   (in-progress) citing zero patterns when
   `[features].patterns-required-severity = "warning"`; observe
   the advisory warning. Run with `severity = "off"` and observe
   the warning silenced.
7. Run `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` — all pass.

---

## TC scaffolding plan

| TC | Type | `observes:` | What it asserts |
|---|---|---|---|
| `author_pattern_session_creates_valid_pat` | session | `[file, graph]` | An `author-pattern` session ends with a PAT on disk that passes `graph check`. |
| `author_feature_surfaces_matching_patterns_by_domain` | scenario | `[stdout]` | A feature authoring session whose `domains:` match an existing PAT exposes the PAT in its prompt context. |
| `feature_link_pattern_writes_bidirectional` | scenario | `[file, graph]` | CLI `product feature link FT-X --pattern PAT-Y` writes both sides atomically. |
| `mcp_feature_link_with_pattern_arg_writes_to_disk` | scenario | `[file, mcp-response]` | MCP `product_feature_link { id, pattern }` produces a file byte-identical to the CLI shape. |
| `mcp_pattern_new_in_authoring_session_writes_to_disk` | scenario | `[file, mcp-response]` | The authoring session's MCP write produces a file on disk; envelope alone is insufficient. |
| `graph_check_advisory_for_feature_with_no_patterns_when_enabled` | scenario | `[stdout, exit-code]` | When `[features].patterns-required-severity = "warning"`, an in-progress feature with empty `patterns:` emits the advisory; `severity = "off"` silences it. |
| `feature_link_pattern_against_deprecated_pat_warns_but_writes` | scenario | `[stdout, file]` | Linking a deprecated PAT emits a stderr warning and writes both sides. |
| `ft_073_exit_criteria_pattern_authoring` | exit-criteria | n/a | Aggregator; cargo gates green; the `author-pattern` prompt is registered and returned by `product_prompts_get`. |
