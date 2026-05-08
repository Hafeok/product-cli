---
id: FT-063
title: Per-Model Context Bundle Templates
phase: 5
status: complete
depends-on:
- FT-011
- FT-027
- FT-021
adrs:
- ADR-049
- ADR-006
- ADR-013
- ADR-020
- ADR-043
- ADR-047
- ADR-018
- ADR-040
- ADR-041
- ADR-042
- ADR-048
tests:
- TC-742
- TC-743
- TC-744
- TC-745
- TC-746
- TC-747
- TC-748
- TC-749
- TC-750
- TC-751
- TC-752
- TC-753
- TC-754
- TC-755
- TC-756
- TC-757
- TC-758
- TC-759
- TC-760
- TC-761
- TC-762
- TC-763
- TC-764
- TC-765
- TC-766
- TC-767
domains:
- api
- observability
domains-acknowledged:
  observability: 'FT-063''s observability surface is thin and bounded: a token-count approximation, two boolean budget-exceeded flags in the MCP response, and one stderr deprecation note for `--for-llm`. The foundational observability ADRs that preflight surfaces — ADR-036 (Tag-Based Implementation Tracking) and ADR-023 (Drift Detection — Spec vs. Implementation) — govern build/CI-time signals about specs and code, not runtime telemetry from `product context`. ADR-040 (Unified Verify Pipeline) is already linked and covers the only observability-adjacent path that does apply (verify-time emission). Linking ADR-036 / ADR-023 would create false signal; the gap is acknowledged rather than papered over.'
---

## Description

Replace the single-format `--for-llm` flag on `product context` with a
general `--target NAME` mechanism backed by per-model **data templates**.
A template is a TOML file declaring structural format (XML, Markdown,
YAML, JSON, plain), section ordering, and informational token-budget
hints for a specific model. Six built-in templates ship with Product
covering Claude Opus 4.7, Claude Haiku 4.5, GPT-4o / GPT-4 Turbo,
GPT-3.5 / GPT-4o-mini, Gemini 2.5, and a `human` terminal-readable
default.

ADR-049 locks three sub-decisions:

1. Templates are data, not code (no template language, no logic).
2. Built-in templates ship as files under `$PRODUCT_INSTALL/templates/`,
   not embedded in the binary.
3. Summarisation is **out of scope** for v1 — templates choose format
   and ordering, not body compression.

The bundle's *content* (which features, ADRs, TCs are included at what
depth) is unchanged by the template. The template controls *how* that
content is rendered. ADR-006's "complete and accurate bundle" invariant
is preserved.

## Depends on

- **FT-011** — Context Bundle Format. Owns the assembly pipeline this
  feature wraps a rendering layer around.
- **FT-027** — Context Bundle. Owns the live `product context` command
  and its existing rendering behaviour (the template layer slots in
  before the final string emission).
- **FT-021** — MCP Server. Owns the `product_context` MCP tool that
  gains the `target` parameter.

## Functional Specification

### Inputs

- **`product context FT-XXX --target NAME`** — required `FT-XXX`,
  optional `--target NAME`. When omitted, falls back to
  `[context].default-target` from `product.toml`, then to `human` if
  unset.
- **`product context templates`** — list resolved templates.
- **`product context templates --show NAME`** — print a template's TOML
  content to stdout.
- **`product context templates --where`** — show resolution path for
  each template.
- **`product context templates --reset NAME`** — remove a user override
  at `~/.product/templates/NAME.toml`.
- **`product_context` MCP tool** — gains `target: string` parameter.
- **Template files** — TOML in three locations searched in order:
  `.product/templates/` (repo) → `~/.product/templates/` (user) →
  `$PRODUCT_INSTALL/templates/` (built-in). First match wins.
- **`product.toml`** — gains `[context].default-target = "NAME"`.

### Outputs

- **`product context FT-XXX --target NAME`** — bundle rendered in the
  format declared by the template (XML / Markdown / YAML / JSON /
  plain). Section ordering, deliverables-at-top placement, and
  per-section emphasis follow the template.
- **`product context templates`** — table listing each template name,
  description, and source (built-in / user / repo).
- **`product context templates --show NAME`** — raw TOML.
- **`product context templates --where`** — `name → resolved-path`
  mapping.
- **MCP `product_context`** — JSON envelope `{format, target, content,
  token_count_approx, exceeded_target_max, exceeded_hard_max}`.
- **Stderr deprecation note** — `product context FT-XXX --for-llm`
  emits `Note: --for-llm is a deprecated alias for --target claude-opus`
  on stderr (does not interfere with stdout piping).

### State

- **Read state.** Three template directories (repo, user, install) are
  read on every `product context` invocation. No caching for v1.
- **Write state.** `--reset NAME` deletes one file under
  `~/.product/templates/`. Built-in templates are read-only and never
  modified. The `apply` step is a single `fs::remove_file` with the
  same advisory-lock discipline as other write tools.
- **Config.** `[context].default-target` is read from `product.toml`
  via `ProductConfig::load_from_root`; no new persisted state beyond
  this one key.

### Behaviour

1. **Resolution.** On startup, the template loader walks `.product/templates/`
   → `~/.product/templates/` → `$PRODUCT_INSTALL/templates/`, deduplicates
   by `[template].name`, and produces a `HashMap<String, ResolvedTemplate>`
   tagged with the source directory. Validation runs per template; failures
   are reported as warnings and the offending template is excluded from
   the targets list.
2. **Selection.** `product context FT-XXX --target NAME` looks up `NAME`
   in the resolved map. If absent, return **E027 unknown-target** listing
   available templates. If `--target` is omitted, read
   `[context].default-target` from `product.toml`, falling back to `human`.
3. **Rendering.** The bundle assembly produces a structured `Bundle`
   value (already an internal type behind `product context`). The
   template-aware renderer walks `[ordering].sections` and emits one
   block per recognised section, choosing the structural envelope based
   on `[format].structure`:
   - `xml` — `<context_bundle><task>...</task><feature>...</feature></context_bundle>`
   - `markdown` — `## Task\n...\n## Feature\n...`
   - `yaml` — top-level mapping with one key per section
   - `json` — same shape, JSON-encoded
   - `plain` — Markdown without framing tags (the legacy default).
   `[format.xml].include_attributes`, `[format.xml].empty_section_handling`,
   `[format.markdown].heading_levels`, and `[format.markdown].table_format`
   are honoured per their declared values.
4. **`deliverables_at_top`** — when true, a flat deliverables list is
   emitted as the first section (or as the first child of `<deliverables>`
   in XML form), in addition to whatever the feature body contains.
5. **`critical_first`** — when true, sections are ordered with task
   framing first, spec next, peripheral context last. When false (e.g.
   GPT-4 templates), the natural feature-first order is used.
6. **Section omission.** Sections not listed in `[ordering].sections`
   are omitted entirely from the rendered output. A minimal template
   can include just `task`, `feature`, `test_criteria`.
7. **Token-budget hints.** After rendering, the approximate token count
   (4-chars-per-token heuristic, the same heuristic FT-040 uses) is
   compared against `[token_budget].target_max` and `[token_budget].hard_max`.
   Bundles never get truncated. Exceeding `target_max` emits a stderr
   note; exceeding `hard_max` emits a stderr warning. Both flags are
   surfaced in the MCP response as booleans.
8. **`--for-llm` alias.** The legacy flag is retained, mapped to
   `--target claude-opus`, and emits the stderr deprecation note. Any
   user passing both `--for-llm` and `--target` simultaneously gets
   **E028 conflicting-target-flags**.
9. **`product context templates --reset NAME`** — looks up `NAME` only
   under `~/.product/templates/`, deletes the file if present, errors
   **E029 cannot-reset-builtin** if `NAME` resolves only to a built-in
   path. Repo-local templates are never auto-deleted (those belong to
   the repo, not the user).

### Invariants

- **Templates cannot modify bundle content.** The renderer reads from
  the assembled `Bundle` value; it never re-queries the graph and
  never mutates artifact bodies. Asserted by TC-754 (omits sections),
  TC-755 (orders sections), TC-756 (deliverables-at-top placement) —
  these tests check rendering, not content.
- **Built-in templates are read-only.** `--reset` cannot touch
  `$PRODUCT_INSTALL/templates/`. Asserted by TC-763.
- **Resolution order is repo → user → built-in.** First match wins;
  later sources are ignored. Asserted by TC-747 / TC-748.
- **Token-budget hints are informational.** No bundle is ever
  truncated. Asserted implicitly by every render test (the full
  content is present in the output) and explicitly via the MCP
  response's `exceeded_*` booleans (TC-765).
- **`default-target = "human"` when unset.** Backward-compat invariant:
  `product context FT-XXX` without flags on a fresh repo produces
  Markdown. Asserted by TC-758.
- **Schema-version compatibility.** Templates with `schema_version`
  newer than the binary supports are rejected with a clear upgrade
  message; older versions are accepted with a warning that not every
  option is honoured.

### Error handling

- **E027 unknown-target** — `--target NAME` references a template that
  does not exist in any resolved location. Lists the available targets.
- **E028 conflicting-target-flags** — `--for-llm` and `--target` passed
  together.
- **E029 cannot-reset-builtin** — `--reset NAME` on a name that resolves
  only to a built-in path.
- **E030 invalid-template** — startup validation failure (missing
  required tables, unknown `format.structure`, unknown section names,
  invalid `adrs_ordered_by` / `tcs_ordered_by` values, unsupported
  `schema_version`). Reported as a warning, not a hard error — the
  binary continues to run on other targets. TC-746 asserts the
  offending template is excluded from the listed targets.
- All errors flow through `ProductError` per ADR-013; new codes are
  registered in `src/error.rs` and the ADR-013 table.

### Boundaries

- **In**: read access to template files in three locations; read access
  to `[context]` config; write access to a single `~/.product/templates/`
  file under `--reset`; rendering against the existing `Bundle` value.
- **Out**: bundle assembly (FT-011 / FT-027 already own this); body
  compression / summarisation; per-artifact filtering (depth and
  selection are bundle-assembly concerns); template-language
  interpretation (templates are pure data); auto-detect of target
  model.
- **Caller responsibilities**: scripts that pipe `product context`
  output should expect Markdown by default (the `human` template), or
  explicitly request a target; agents reading the MCP response should
  inspect `format` to decide how to parse `content`.

## Tool surface

### `product context` (extended)

```bash
product context FT-009                        # default target from product.toml
product context FT-009 --target claude-opus   # explicit target
product context FT-009 --target gpt-4-markdown
product context FT-009 --target human         # human-readable Markdown
product context FT-009 --target raw           # no template, plain Markdown (alias for human)
```

### `product context templates` (new)

```bash
product context templates                     # list all available templates
product context templates --show NAME         # print a template's TOML content
product context templates --where             # show resolution path for each
product context templates --reset NAME        # remove user override
```

### `product_context` MCP tool (extended)

```json
// Input
{ "feature_id": "FT-009", "depth": 2, "target": "claude-opus" }

// Output
{
  "format": "xml",
  "target": "claude-opus",
  "content": "<context_bundle ...>...</context_bundle>",
  "token_count_approx": 6840,
  "exceeded_target_max": false,
  "exceeded_hard_max": false
}
```

## Built-in Templates

Six templates ship with Product. Each is a TOML file in
`$PRODUCT_INSTALL/templates/`. Users copy any of them into
`~/.product/templates/` and modify locally.

| Template | Format | Target model | Context window | Notes |
|---|---|---|---|---|
| `claude-opus` | XML | Claude Opus 4.7 | 1,000,000 | full sections incl. `bundle_metrics` |
| `claude-haiku` | XML | Claude Haiku 4.5 | 200,000 | omits `linked_documentation`, `bundle_metrics` |
| `gpt-4-markdown` | Markdown | GPT-4o / GPT-4 Turbo | 128,000 | `critical_first = false` |
| `gpt-mini-json` | JSON | GPT-3.5 / GPT-4o-mini | 128,000 | minimal section set |
| `gemini-yaml` | YAML | Gemini 2.5 Pro | 1,000,000 | full sections |
| `human` | Markdown | n/a | 0 | terminal-readable; default fallback |

Full TOML for each template is reproduced in ADR-049's accompanying
spec file (and committed to `$PRODUCT_INSTALL/templates/` as the
shipping artifact).

## Section names

Recognised section names for `[ordering].sections`:

| Name | Content |
|---|---|
| `task` | Task framing — what the bundle consumer should do with this |
| `feature` | The primary feature artifact (id, metadata, full body) |
| `deliverables` | Flat list of deliverables, duplicated for visibility |
| `governing_adrs` | All ADRs governing the feature, ordered per template |
| `test_criteria` | All linked TCs, ordered per template |
| `dependencies` | All DEPs the feature uses |
| `linked_documentation` | DOC artifacts covering the feature |
| `constraints` | Platform-wide constraints (code quality, verification) |
| `bundle_metrics` | Token count and shape metadata (when `--measure` used) |

Sections not listed in `ordering.sections` are omitted entirely.

## Acceptance criteria

A spec-authoring or implementation agent connected over MCP / CLI can:

1. Run `product context FT-XXX --target claude-opus` and observe an
   XML-structured bundle on stdout (TC-749).
2. Run `product context FT-XXX --target gpt-4-markdown`,
   `--target gemini-yaml`, `--target gpt-mini-json`, `--target human`
   and observe Markdown / YAML / JSON / framing-free Markdown
   respectively (TC-750 / TC-751 / TC-752 / TC-753).
3. Author a custom template in `.product/templates/team-bundle.toml`
   and observe `product context FT-XXX --target team-bundle` use it,
   overriding any user-level template by the same name (TC-747 / TC-748).
4. Run `product context templates`, `--show NAME`, `--where`, `--reset NAME`
   and observe the documented output (TC-759 / TC-760 / TC-761 / TC-762).
5. Run `product context templates --reset claude-opus` against a
   built-in-only resolution and receive **E029** without any file
   being deleted (TC-763).
6. Submit a malformed template (missing required tables, invalid
   `format.structure`, unknown section name) and observe it excluded
   from the targets list with a startup warning (TC-743 / TC-744 /
   TC-745 / TC-746).
7. Set `[context].default-target = "claude-opus"` in `product.toml`
   and observe `product context FT-XXX` (no flag) produce the XML
   bundle (TC-757); leave it unset and observe the `human` Markdown
   default (TC-758).
8. Call `product_context` over MCP with `target: "claude-opus"` and
   receive `{format: "xml", target: "claude-opus", ...}` in the
   response (TC-764 / TC-765).
9. Run `product context FT-XXX --for-llm` and observe the same bundle
   `--target claude-opus` produces, plus a stderr deprecation note
   (TC-766).
10. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
    and `cargo build` all pass.
11. `product graph check` exits clean with the new templates resolved.

## Out of scope

- **Body summarisation / compression.** Templates choose format and
  ordering. Body compression is a separate problem (model-specific
  tokenisation, prompt-injection risk, mismatch with `product feature
  show`) and is explicitly out of scope for v1.
- **Filtering which artifacts get included.** Depth and artifact
  selection are bundle-assembly concerns owned by FT-011 / FT-027.
  Templates do not see the graph; they see the assembled `Bundle`.
- **Custom section names.** The recognised section list is closed.
  Adding a new section requires a Product release and an additive
  schema-version bump.
- **Template language / logic / loops / conditionals.** Templates are
  pure data. A language would push us across the safety boundary that
  motivates the file-not-code decision.
- **Auto-detect target from environment / model name.** Routine
  prompts (ADR-022 system prompts) are the right place to specify
  target; auto-detect is too lossy.
- **`product context templates --create` scaffold command.** Copy
  and edit is the supported workflow for v1. A scaffold may follow if
  there's demand.
- **`--max-tokens` truncation.** Already rejected by ADR-006 and
  re-rejected by ADR-049; templates' `token_budget` keys are warnings
  only.

## Implementation notes

- **`src/context/template/`** (new slice). `mod.rs` re-exports the
  loader, validator, resolver, and renderer. `loader.rs` reads TOML
  from the three locations. `validate.rs` runs the closed-allowlist
  checks and returns findings. `resolve.rs` produces the merged
  `HashMap<String, ResolvedTemplate>`. `render.rs` walks the assembled
  `Bundle` and emits XML / Markdown / YAML / JSON / plain. Pure
  functions throughout per ADR-043.
- **`src/commands/context.rs`** — extend the existing handler to
  accept `--target NAME`, fetch the template, route through the new
  renderer. The `templates` subcommand is a separate adapter.
- **`src/mcp/read_handlers.rs::handle_context`** — accept the optional
  `target` parameter, forward to the slice, format the response with
  the new fields.
- **Built-in templates** ship as files under `templates/` at the
  repo root, installed to `$PRODUCT_INSTALL/templates/` by the
  package manager / install script. The fitness suite's file-length
  cap (400 lines) does not apply to TOML data files — confirmed.
- **Schema-version handling.** `schema_version = 1` is the only
  supported value at v1. The validator rejects anything else with
  a clear upgrade message.
- **Error codes.** `E027` (unknown-target), `E028`
  (conflicting-target-flags), `E029` (cannot-reset-builtin), `E030`
  (invalid-template) registered in `src/error.rs` and the ADR-013
  table.
- **Routine update.** Step 3 of `benchmarks/prompts/implement-v1.md`
  changes from `product context FT-XXX --depth 2 --for-llm` to
  `product context FT-XXX --depth 2 --target claude-opus`. The
  routine ships configured for `claude-opus` because that's what
  Anthropic's cloud runs the routines on; users on different harnesses
  update the target value in their copy.
- **Runner config.** Every TC in this feature gets `runner: cargo-test`
  and `runner-args: "tc_NNN_snake_case"` at the same time the test is
  written, per CLAUDE.md.

## Migration from `--for-llm`

The `--for-llm` flag becomes a deprecated alias for `--target
claude-opus`. Existing scripts and routines continue to work but emit
a stderr deprecation note:

```
$ product context FT-009 --for-llm

  Note: --for-llm is a deprecated alias for --target claude-opus.
        Update your scripts to use --target NAME explicitly.

[bundle output...]
```

The deprecation note is on stderr so it does not interfere with
piping. The flag will be removed in a future schema-version bump
following the ADR-014 deprecation cycle.

## Effect on the routine

Step 3 of `implement-next-feature.md` becomes:

```
## Step 3 — Get full context for the feature

Call `product_context FT-XXX --depth 2 --target claude-opus`.

The `--target claude-opus` parameter selects the XML-structured rendering
optimised for Claude Opus 4.7. If you are running on a different model,
adjust the target accordingly:

  - Claude Opus 4.7  →  --target claude-opus
  - Claude Haiku 4.5 →  --target claude-haiku
  - GPT-4o           →  --target gpt-4-markdown
  - Gemini 2.5 Pro   →  --target gemini-yaml

Read the entire bundle. ...
```

The routine ships configured for `claude-opus` because that's what
Anthropic's cloud runs the routines on. If a user runs the routine
through a different harness with a different model, they update the
target value in their copy of the routine prompt.
