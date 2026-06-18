---
id: FT-117
title: product cell dispatch — instantiate a task type into frozen SPMC work units
phase: 6
status: complete
depends-on:
- FT-113
- FT-116
adrs:
- ADR-059
tests:
- TC-970
- TC-971
- TC-972
- TC-973
- TC-974
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `cell dispatch` verb; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Followed — the instantiation logic lives in the pure `pf::dispatch` slice; the CLI is a thin BoxResult adapter.
  ADR-048: Reads a task-type YAML + the What session; writes work units under `.product/work-units/`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Five scenario TCs drive the binary through the assert_cmd harness; the pf::dispatch slice carries unit tests. No property or session dimension for a deterministic instantiator.
  ADR-040: Dispatch is a structural transformation (template → frozen manifests); it runs no model and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product cell dispatch` is the realisation step: it turns a task type (a
template, FT-113) plus slot bindings into the concrete, frozen SPMC work units
(FT-116) it produces — resolving each cell's `domain:<slot>` references to the
bound domain entities and freezing the context. This is where the How's
realisation layer actually touches the captured What graph.

It closes the framework loop: **What** (captured entities) → **How**
(task-type cells applying patterns) → **dispatch** → concrete work units that
themselves validate against the What graph and the How contract.

## Functional Specification

### Inputs

- A task-type YAML (`--file`, default `.product/cell.yaml`).
- `--bind slot=value` (repeatable) — the slot bindings.
- The default product's What graph (`--product` to override) to verify entity
  bindings.
- `--out <dir>` (default `.product/work-units/`) or `--print` (stdout only).

### Behaviour

- Validates bindings: every binding names a declared slot; every required slot
  is bound; any slot a cell derives from via `domain:<slot>` must bind to a
  value that exists as an entity in the What graph.
- For each cell, instantiates a work unit: resolves `derived_from` slot
  pointers to the bound values (`domain:entity` → `domain:Order`), freezes and
  content-hashes the context, synthesises a single-purpose prompt, sets the
  produced artifact, and emits a rationale trace (`what` = bound entity, `why`
  = applied patterns).
- Writes each work unit to `<out>/<id>.yaml` (ids keyed by the bound entity),
  or prints them with `--print`. The produced units pass `work-unit validate`.

### Error handling

- A binding to a non-entity, a missing required slot, or a binding naming no
  declared slot is a blocking violation (exit 1); nothing is instantiated.
- A malformed `--bind` (no `=`) is a clear error.

## Out of scope

- It does not execute the SPMC prompt (run a model to emit the artifact) —
  dispatch produces the frozen manifests; execution is future work.
- It does not re-validate the task type itself (`product cell validate` does).

## Acceptance

- TC-970 — dispatch instantiates frozen work units with resolved domain pointers.
- TC-971 — the dispatched work units pass `work-unit validate`.
- TC-972 — binding to a non-entity is rejected (nothing written).
- TC-973 — a missing required binding is rejected.
- TC-974 — `--print` emits to stdout without writing files.
