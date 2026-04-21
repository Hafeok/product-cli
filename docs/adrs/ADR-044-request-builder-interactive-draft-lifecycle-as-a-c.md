---
id: ADR-044
title: Request Builder — Interactive Draft Lifecycle as a Composition Layer over ADR-038
status: proposed
features:
- FT-052
supersedes: []
superseded-by: []
domains:
- api
- data-model
- error-handling
scope: domain
---

**Status:** Proposed — amends ADR-038

**Context:** ADR-038 pins the unified Product Request as the single
composable write interface. A request is a YAML document describing
intent (artifacts to create, mutations to apply) that is validated
as a whole and applied atomically. That interface is the agent
path: an LLM reasons about complete intent and emits complete YAML.

Humans at a terminal do not work that way. A human authoring a
feature + ADR + TC + DEP graph iterates — add one artifact, see
what Product thinks, add the next. Forcing them to produce the full
YAML up-front loses the feedback loop and turns authoring into
edit-validate-edit ping-pong against a single file. The two cases
that matter in practice:

1. **Discovery** — the human does not know the final shape. They
   declare a feature, then realise it needs two ADRs, then a TC,
   then a DEP. Each step benefits from immediate structural
   validation (domain vocabulary, scope enum, tc-type) against the
   *partial* draft plus the existing graph.
2. **Cross-artifact scaffolding** — common patterns (dep → governing
   ADR; feature → domain acknowledgement) should be one prompt, not
   two disconnected artifact authorings. E013 and W010 want to
   close in the same keystroke that could have created them.

The existing single-shot `product request apply FILE` path satisfies
neither. An interactive wrapper that speaks the exact same YAML —
with no builder-only features, no server-side draft state, and no
divergence in validation rules — closes the gap.

**Decision:** Introduce an interactive request builder as a thin
incremental editor on top of ADR-038's request YAML. The draft is a
plain request YAML file at `.product/requests/draft.yaml`. Every
`product request add …` command appends one artifact or one
mutation to that file and runs the same validator ADR-038 defines,
scoped to the new content. `product request submit` is exactly
`product request apply .product/requests/draft.yaml` plus archive.

### Decisions pinned by this ADR

1. **The draft is request YAML; nothing else.** No builder metadata,
   no sidecar state file, no server-side draft store. Opening
   `.product/requests/draft.yaml` in an editor or passing it to
   `product request apply` yields identical behaviour to submit.
2. **One active draft per working directory.** `product request new`
   with an existing draft surfaces it (status / submit / discard /
   continue) rather than silently overwriting. The lockfile is the
   draft file's existence.
3. **Incremental validation is structural only.** After each `add`,
   validation runs against the draft + the existing graph — schema,
   vocabulary, ref resolution, E013 closure within the draft — and
   must complete in under 100ms. No LLM, no git, no cross-apply
   simulation. The full `product request validate` remains the
   pre-submit gate.
4. **Flags skip prompts; prompts and flags produce identical YAML.**
   `product request add feature --title X --phase 2 …` and the
   interactive flow both append the same artifact block. Scripting
   parity is a non-negotiable — the builder is a thin UX on top of
   YAML, not a richer authoring mode.
5. **`add dep` is allowed to add a governing ADR in the same step.**
   The `--adr new` option creates a ref-linked ADR in the same
   append, satisfying E013 before it can fire. Linking to an
   existing ADR validates existence and no-other-governor.
6. **`add target` on change mode suggests follow-up mutations.**
   When a mutation would newly trigger a W-class finding (adding a
   domain without an acknowledgement, W010), the builder prompts to
   add the follow-up mutation in the same step. Suggestions are
   never automatic; the human confirms each mutation.
7. **Submit archives the draft.** On success, the draft moves to
   `.product/requests/archive/<timestamp>-draft.yaml`. The archive
   directory is gitignored — local history, not shared artefact.
   Failed submit leaves the draft in place unchanged.
8. **No builder-only capability.** Every `add` / `target` operation
   maps 1:1 to a YAML construct expressible in a hand-written
   request. The builder is a convenience surface; removing it
   would not reduce what the request model can express.
9. **`product request edit` opens `$EDITOR` on the draft.** For
   users who prefer to write YAML directly but want the lifecycle
   management (draft tracking, archive on submit).
10. **`warn-on-warnings` config controls submit behaviour.** Values
    `always` (submit through warnings), `warn` (prompt — default),
    `block` (treat W-class as E-class). Non-interactive `--force`
    submits through warnings without prompting.

**Rationale:**

- **The draft *is* the YAML.** No dual representation means no
  divergence between what the builder produces and what the direct
  YAML path accepts. The `builder-output-identical-to-hand-written`
  invariant is free by construction.
- **Incremental validation closes feedback loops.** A human adding
  a dep learns about the governing-ADR requirement at the moment
  they add it, not after a 5-artifact submit fails. Structural-only
  scope keeps the feedback under 100ms.
- **Suggestions over automation.** The builder suggests
  acknowledgements and cross-links but never writes them without
  confirmation. This preserves the "intent as data" promise from
  ADR-038 — every mutation in the final YAML was chosen by a human.
- **Archiving is local history, not audit.** The
  `.product/request-log.jsonl` hash chain (ADR-039) is the audit
  record; the draft archive is the builder's own undo surface. Kept
  gitignored to avoid polluting repository diffs with session
  state.

**Rejected alternatives:**

- **Server-side draft store with request IDs.** Would enable
  collaboration on a single draft across agents, but duplicates
  state the filesystem already holds and introduces a registry
  that needs its own consistency story. Rejected — the draft file
  IS the draft.
- **Distinct in-memory draft model with a YAML exporter.** Tempting
  because it would let the builder accumulate rich typed state,
  but it breaks the "draft is the YAML" invariant — the exporter
  could drift from the validator's input model. Rejected.
- **Auto-acknowledge on domain add.** The builder could silently
  set `domains-acknowledged.<domain>` with a templated reason
  whenever a feature adds a domain without a governing ADR.
  Rejected because the reason field is the whole point of the
  acknowledgement — a machine-generated placeholder is worse than
  no acknowledgement.
- **LLM-assisted validation in the incremental loop.** Some checks
  (cross-artifact consistency, narrative coherence) would benefit
  from LLM review after each `add`. Rejected for the incremental
  path because of latency and because gap-class analysis already
  runs pre-submit under `product request validate`.

### Test coverage

| Decision | Covered by TC (title) |
|---|---|
| Draft IS the YAML | `builder-draft-file-is-request-yaml` |
| One active draft | `builder-new-with-existing-draft-surfaces-options` |
| Incremental validation scope | `builder-add-runs-structural-validation-only` |
| Flags = prompts parity | `builder-flags-and-prompts-produce-identical-yaml` |
| `add dep` closes E013 in-step | `builder-add-dep-with-new-adr-satisfies-e013` |
| `add target` suggests follow-ups | `builder-target-suggests-acknowledgement-on-w010` |
| Submit archives on success | `builder-submit-archives-draft` |
| Submit blocked on E-class | `builder-submit-blocked-on-e-class-errors` |
| Identical semantics to apply | `builder-submit-applies-identically-to-request-apply` |
