---
id: ADR-077
title: The work-unit path is mandatory and harness-authoritative; cells template it per slot
status: accepted
features:
- FT-132
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
content-hash: sha256:a94cd40e77904a5c7c20c8ca354d67453cb2d04a236ddd7579d2086222ba0c58
source-files:
- product-core/src/pf/work_unit.rs
- product-core/src/pf/cell.rs
- product-core/src/pf/dispatch.rs
- product-core/src/pf/worker.rs
---

## Context

The worker's output contract let the *model* choose placement: it returned
`{"files":[{"path","content"}]}` and the harness wrote to whatever path the model
emitted. `produces.path_hint` was optional and advisory — prompt guidance and the
guard's allow-set, but never authoritative. So an under-specified worker that
hallucinated a path (`src/commands/<thing>.rs` for a workspace that has none)
scattered whole file trees in the wrong place.

The spec-depth ablation makes the lesson precise: *localization is the one thing
no model size recovers* — at L1 every tier, including the largest, hallucinated
placement. Letting the model own the thing it provably gets wrong is the bug.

## Decision

- **`produces.path` is required and concrete** (renamed from the optional
  `path_hint`). A work unit declares the exact path its one artifact lands at;
  validation rejects an empty path (§5 — one unit, one artifact, one place).
- **The harness owns placement.** `worker::retarget` forces the worker's output to
  the declared path and keeps a single produced file; `dispatch_to` threads the
  unit's `produces.path` through, so a hallucinated or extra path is ignored. We
  read the model for *content*, never for *location*.
- **Cells template the path.** A cell's `path` (or `edits`) is a template that may
  reference the task-type's slots as `<slot>`; `dispatch` resolves it against the
  bindings (`resolve_path`) into the concrete work-unit path — the same dual-read
  move already used for `derived_from`. So a cell stays a reusable *pattern*
  (`src/pf/<concept>.rs`) while each dispatched work unit is concrete
  (`src/pf/order.rs`).

## Rationale

- Prevention over cleanup: the oracle guard (ADR-076) reverts stray writes *after*
  the fact; making placement harness-owned removes the failure class up front and
  leaves the guard as defense-in-depth.
- The cell path template answers "how do patterns stay reusable when units are
  concrete": the literal filename lives nowhere — the cell names a path *shape*
  parameterised by a slot, and dispatch fills it per feature.
- Reusing the existing slot-substitution keeps the model and the schema small; no
  new resolution machinery.

## Rejected alternatives

- **Keep model-chosen paths, rely on the guard to revert strays.** Rejected:
  reverting hallucinations is cleanup; not offering the model the choice is the fix.
- **A path-less content contract (`{content}` / `{edit}`).** Rejected: a larger
  prompt/schema change that also drops multi-file capability — ignoring the model's
  `path` field achieves the same with the existing JSON contract.
- **Concrete per-cell file names.** Rejected: a cell would then bind to one
  feature; the slot template is what keeps it a pattern.

## Test coverage

- `pf::dispatch` — a templated cell path resolves to a concrete per-binding path.
- `pf::worker` — `retarget` forces a wrong (and an extra) path to the single
  declared path; every edit is pointed at it.
- `pf::work_unit_validate` — a work unit must declare a non-empty `produces.path`.
