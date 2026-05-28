---
id: FT-052
title: Product Request Builder — Interactive Draft Sessions
phase: 5
status: complete
depends-on:
- FT-038
- FT-041
adrs:
- ADR-038
- ADR-044
tests:
- TC-626
- TC-627
- TC-628
- TC-629
- TC-630
- TC-631
- TC-632
- TC-633
- TC-634
- TC-635
domains:
- api
- data-model
- error-handling
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: The builder does not introduce absence TCs and does not interact with ADR removes/deprecates lifecycle fields; draft artifacts are ordinary create/change requests against the existing schema.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-043: 'Implementation follows the slice + adapter pattern: a new `src/request/builder/` slice with pure `plan_*` planners and a thin `src/commands/request.rs` dispatch extension — no deviation from ADR-043.'
  ADR-040: The builder reuses the existing `product request apply` pipeline verbatim for submit; it adds no new stage to the unified verify pipeline and no new hooks at the LLM boundary — incremental validation is structural-only and intentionally stays off the LLM side of the knowledge boundary.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-042: Consumed unchanged — when the user runs `add tc`, the builder validates `tc-type` against the ADR-042 structural/built-in-descriptive/custom partition; it does not extend the partition or introduce new TC types.
---

## Description

The unified Product Request interface (FT-041, ADR-038) is the
single composable write surface. Today it has one shape: write the
full YAML, validate it, apply it. That is the agent path. For a
human at a terminal authoring a feature alongside its governing
ADRs, TCs, and DEPs, the edit-validate-edit loop across a single
growing YAML file is hostile — the feedback arrives only at
submit, and common cross-artifact patterns (dep + governing ADR;
feature + domain acknowledgement) require remembering to emit both
sides of the relationship.

This feature introduces an interactive request builder that turns
the same request YAML into an incremental session. A draft is
opened, artifacts are added one at a time with immediate
structural validation, the current state is inspectable, and
submit is exactly `product request apply draft.yaml`. The draft
file IS the YAML — opening it in `$EDITOR` or piping it to
`product request apply` at any point produces identical results.

The full builder surface is specified in
[`docs/product-request-builder-spec.md`](/docs/product-request-builder-spec.md);
the pinned decisions live in the governing ADR.

---

## Depends on

- **FT-041** — the unified request interface this feature wraps.
  Submit is `product request apply` on the draft file.
- **FT-038** — the granular front-matter mutation tools whose
  validation rules (domain vocabulary, scope enum, runner values)
  the incremental validator reuses.

---

## Scope of this feature

### In

1. **Draft lifecycle.** `product request new create|change`
   creates `.product/requests/draft.yaml`. `product request
   continue` resumes an existing draft. `product request discard`
   deletes it. One active draft per working directory — starting
   a new session when a draft exists surfaces status / submit /
   discard / continue options rather than overwriting.
2. **`add` commands — create mode.** `add feature`, `add adr`,
   `add tc`, `add dep`, `add doc` each prompt for required fields
   (or accept all fields as flags), append one artifact block to
   the draft, and run incremental structural validation against
   the draft + existing graph. Assigned `ref:` name is returned
   for use in subsequent steps.
3. **`add target` — change mode.** Adds a target artifact and
   opens an interactive mutation builder: append / set / remove /
   delete one field at a time. After each mutation, the builder
   re-validates and suggests follow-up mutations (e.g. adding a
   domain without acknowledgement prompts to add the
   acknowledgement in the same session).
4. **`add acknowledgement` shortcut.** `add acknowledgement ID
   DOMAIN REASON` — one-shot form for the common W010 closure
   case.
5. **`status` / `show` / `validate` / `diff`.** `status` renders a
   human-readable summary with ✓ / ⚠ / ✗ indicators per artifact
   and a warning / error count. `show` prints the raw draft YAML.
   `validate` runs the full cross-artifact pass (identical to
   `product request validate draft.yaml`). `diff` shows what
   would change on submit.
6. **`submit` / `edit`.** `submit` validates then applies the
   draft atomically, archives it on success to
   `.product/requests/archive/<timestamp>-draft.yaml`, and prints
   assigned IDs. Refuses to apply on any E-class finding. With
   W-class findings, respects `warn-on-warnings`. `edit` opens
   the draft in `$EDITOR` directly — useful for users who prefer
   raw YAML with the builder's lifecycle management.
7. **`product.toml` config.** New `[request-builder]` section:
   `interactive = true` (prompts when stdin is a tty),
   `warn-on-warnings = "warn" | "always" | "block"`, optional
   `editor` override.
8. **Unit + integration tests.** At minimum: draft lifecycle
   end-to-end, each `add` subcommand's validation, status /
   validate output formats, submit happy path and error paths,
   YAML equivalence with hand-written requests.

### Out

- **Multi-draft sessions per working directory.** One draft at a
  time; the spec's `continue` command is the resume path.
- **Server-side draft state.** The draft is purely a file on disk
  under `.product/requests/`. No request IDs, no registry.
- **Cross-machine draft sync.** The draft is gitignored and
  per-clone; the spec is explicit that archives are local history.
- **Teaching the request schema in the prompts.** Prompts ask for
  field values; they do not explain why the field exists. The
  authoritative schema source is `product schema` and the request
  spec.
- **Deleting artifacts via the builder.** ADR-038 decision 1: the
  request model does not support deletion; this feature inherits
  that restriction.

---

## Commands

Session management:
```
product request new create | change
product request continue
product request discard [--force]
```

Building (create mode):
```
product request add feature [FLAGS]
product request add adr [FLAGS]
product request add tc [FLAGS]
product request add dep [FLAGS]
product request add doc [FLAGS]
```

Building (change mode):
```
product request add target ID [FLAGS]
product request add acknowledgement ID DOMAIN REASON
```

Inspection & submission:
```
product request status | show | validate | diff
product request submit [--force]
product request edit
```

---

## Implementation notes

- **`src/request/builder/`** — new slice. `plan_new`, `plan_add_*`,
  `plan_submit` as pure planners returning draft mutations; thin
  `apply_*` wrappers call the existing `fileops::write_file_atomic`
  and the existing `product request apply` entry point for submit.
- **`src/commands/request.rs`** — extend the existing dispatch
  with `new`, `add`, `status`, `submit`, `edit`, `discard`,
  `continue`. Handlers stay thin — the slice owns the planning.
- **Validation reuse.** The incremental validator is the existing
  `request::validate` scoped to the newly-added artifact plus
  existing graph. No parallel validation code path — one validator,
  one schema.
- **Archive rotation.** Simple timestamped filenames; no retention
  policy in v1. If the archive grows large, a future feature adds
  a `product request archive prune` command.
- **File-length budget.** The spec is long; the implementation
  must respect the 400-line-per-file limit. Expect to split the
  builder slice across `draft.rs`, `add.rs`, `status.rs`,
  `submit.rs`, `render.rs`.
- **No new dependencies.** Pure reuse of existing YAML, atomic
  write, and lock primitives.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Run `product request new create`, then
   `product request add feature` with required fields, then
   `product request status`, and observe the draft contains the
   feature with a `ref:` name and status indicators.
2. Run `product request add dep --adr new …` in a draft and
   observe that both a DEP and its governing ADR are appended to
   the draft in one step, satisfying E013 within the draft.
3. Run `product request submit` on a draft that applies cleanly
   and observe:
   - The files are written with resolved real IDs
   - The draft is archived under `.product/requests/archive/`
   - The request log gains one entry with the draft's `reason:`
4. Attempt `product request submit` on a draft with an E-class
   finding and observe the submit is refused, the draft is left
   in place, and zero files are written.
5. Run `product request new create` while a draft exists and
   observe the builder surfaces status / submit / discard /
   continue options rather than overwriting.
6. Author a request in `$EDITOR` via `product request edit`, run
   `product request validate`, and observe the same findings as a
   prompted session produces for the same intent (YAML
   equivalence invariant).
7. Run `cargo test`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` and observe all pass.

See the exit-criteria TC for the consolidated check-list.

---

## Follow-on work

- **Archive pruning.** A `product request archive prune` command
  to cap `.product/requests/archive/` size. Deferred.
- **Session transcript export.** Persist each `add` command's
  prompts + answers alongside the archived draft for auditability
  beyond the YAML. Deferred — the YAML is sufficient for v1.
- **Multi-draft workspaces.** Named drafts (`product request new
  create --name rate-limit`) for concurrent exploration. Deferred;
  current one-draft rule keeps the UX simple.

---

## Functional Specification

### Inputs

- **`product request new create|change`** — no arguments required; mode (`create` or `change`) determines which `add` subcommands are available in the session.
- **`product request add <artifact> [FLAGS]`** — accepts all required artifact fields as flags (e.g. `--title`, `--phase`, `--scope`) or prompts for them interactively when stdin is a TTY. Flags and prompts produce byte-identical YAML (ADR-044 decision 4).
- **`product request add target ID [FLAGS]`** — change mode only; targets an existing artifact by ID and opens an interactive mutation builder (append / set / remove / delete one field at a time).
- **`product request add acknowledgement ID DOMAIN REASON`** — one-shot form for closing W010 domain-acknowledgement findings within a draft session.
- **`product request submit [--force]`** — reads the current `.product/requests/draft.yaml`; applies it via the existing `product request apply` pipeline; archives on success.
- **`product request continue`** — resumes an existing draft; prints its status summary.
- **`product request discard [--force]`** — deletes the draft file; prompts for confirmation unless `--force` is given.
- **`product request edit`** — opens `.product/requests/draft.yaml` in `$EDITOR`.
- **`product.toml` `[request-builder]` section** — `interactive`, `warn-on-warnings`, and optional `editor` override.

### Outputs

- **`.product/requests/draft.yaml`** — the draft file, which is valid request YAML at every step. Opening it in `$EDITOR` or passing it directly to `product request apply` produces identical results to using the builder's `submit` command (ADR-044 decision 1).
- **`product request status` / `show` / `validate` / `diff` output** — human-readable summary with status indicators (✓ / ⚠ / ✗) per artifact, warning/error counts, raw draft YAML, or a diff of what would change on submit.
- **Archived draft** — on successful submit, the draft is moved to `.product/requests/archive/<timestamp>-draft.yaml`. The archive directory is gitignored (local history, not shared artifact).
- **Committed graph writes** — `product request submit` delegates to the existing `product request apply` pipeline, which performs all graph writes atomically and appends one entry to `requests.jsonl`.
- **Incremental validation feedback** — after each `add` command, structural validation runs against the draft plus the existing graph and prints any E-class or W-class findings immediately.

### State

The single persistent artifact is `.product/requests/draft.yaml`. Its existence is the lockfile for the "one active draft per working directory" rule (ADR-044 decision 2): `product request new` with a draft present surfaces status / submit / discard / continue options rather than overwriting. No server-side draft store, no sidecar state file, and no registry — the draft file IS the session state.

The archive directory (`.product/requests/archive/`) accumulates timestamped copies of submitted drafts as local history. It is gitignored and carries no retention policy in v1.

### Behaviour

1. **Session open.** `product request new create` creates `.product/requests/draft.yaml` with a `type: create` header. `product request new change` creates one with `type: change`. If a draft already exists, the command prints its status and offers continue / submit / discard / new-session choices (ADR-044 decision 2).
2. **Incremental artifact addition.** Each `add` command appends one artifact block (create mode) or one mutation target (change mode) to the draft YAML, then runs structural validation scoped to the newly-added content plus the existing graph. Validation must complete in under 100ms — no LLM, no git, no cross-apply simulation (ADR-044 decision 3).
3. **`add dep` closes E013 in-step.** `product request add dep --adr new …` appends both the DEP and its governing ADR in one step, satisfying E013 before it can fire (ADR-044 decision 5).
4. **Follow-up mutation suggestions.** When a mutation in change mode would newly trigger a W-class finding (e.g. adding a domain without an acknowledgement triggers W010), the builder prompts to add the follow-up in the same step. Suggestions are never automatic — the user confirms each mutation (ADR-044 decision 6).
5. **Submit.** `product request submit` runs the full `product request validate` pass, refuses to apply on any E-class finding, and respects `warn-on-warnings` for W-class findings. On success, files are written atomically, the draft is archived, and assigned IDs are printed. On failure, the draft is left in place unchanged (ADR-044 decision 7).
6. **Edit shortcut.** `product request edit` opens the draft in `$EDITOR` directly — the full YAML is exposed for users who prefer raw editing but still want the lifecycle management.
7. **YAML equivalence.** Every `add` command invoked with all fields as flags produces the same draft YAML as the interactive prompt flow for the same field values (ADR-044 decision 4, TC-634).

### Invariants

- The draft file is always valid request YAML that can be passed to `product request apply` at any point and produce the same result as `product request submit` (TC-634).
- At most one draft exists per working directory at any time. `product request new` with an existing draft never overwrites it silently (ADR-044 decision 2, TC-632).
- `product request submit` on a draft with an E-class finding writes zero files to disk and leaves the draft untouched (TC-631).
- Flags-only invocation and interactive-prompt invocation of the same `add` command produce byte-identical YAML (ADR-044 decision 4).
- Archive is gitignored; no draft archive entry ever appears in `git status` as an untracked file for commit (ADR-044 rationale).
- Incremental validation uses the identical validator and schema as `product request validate` — there is no separate validation code path (ADR-044 decision 3).

### Error handling

- **E-class finding at submit.** Submit is refused; the draft is left in place; all E-class findings are printed with ADR-013-style diagnostics. Exit 1.
- **W-class finding at submit.** Behaviour depends on `warn-on-warnings`: `always` submits through, `warn` prompts (default), `block` treats W-class as E-class. `--force` overrides without prompting.
- **`product request new` with existing draft.** Not an error — the builder surfaces the existing draft's status and the available options (continue / submit / discard). No overwrite occurs.
- **`product request add` with missing required field (no-TTY, no flag).** E-class finding; the partial artifact is not appended to the draft; the draft is left in its prior state.
- **`product request discard` without `--force`.** Prompts for confirmation before deleting. If stdin is not a TTY, requires `--force`; without it, exits with an error.
- **`$EDITOR` not set on `product request edit`.** Falls back to `vi`; emits a W-class warning suggesting the user configure `$EDITOR` or the `[request-builder].editor` override.

### Boundaries

- **In scope:** the draft lifecycle (new / continue / discard / edit), the `add` family of commands (feature, adr, tc, dep, doc, target, acknowledgement), the inspection surface (status / show / validate / diff), and the submit / archive flow.
- **Out of scope:** multi-draft sessions per working directory. One draft at a time; named drafts are deferred.
- **Out of scope:** server-side draft state or request IDs. The draft is purely a file on disk under `.product/requests/`.
- **Out of scope:** cross-machine draft sync. The draft is gitignored and per-clone.
- **Out of scope:** artifact deletion via the builder. ADR-038 decision 1 — the request model does not support deletion; this feature inherits that restriction.
- **Out of scope:** LLM-assisted incremental validation. Incremental validation is structural-only; gap-class analysis runs only via `product request validate` at the full-submit gate.

## Out of scope

- **Multi-draft workspaces.** Named drafts (`product request new create --name rate-limit`) for concurrent exploration. Deferred — the one-draft-at-a-time rule keeps the UX simple and the lockfile trivial.
- **Server-side draft state.** No request IDs, no draft registry. The draft file IS the session; collaboration on a shared draft is out of scope (ADR-044 rejected alternatives).
- **Cross-machine draft sync.** The archive is gitignored and per-clone. Shared draft state would require a registry that introduces its own consistency story.
- **Teaching the request schema in prompts.** Prompts ask for field values; they do not explain why each field exists. The authoritative schema source is `product schema` and the request spec.
- **Deleting artifacts via the builder.** ADR-038 decision 1: the request model does not support deletion; this feature inherits that restriction unchanged.
- **Archive pruning in v1.** A `product request archive prune` command is deferred; no retention policy ships in this feature.
- **Session transcript export beyond YAML.** Persisting each prompt + answer alongside the archived draft for auditability beyond the YAML is deferred — the YAML is sufficient for v1.
