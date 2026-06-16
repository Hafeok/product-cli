---
id: ADR-059
title: Cell dispatch instantiates a task type into frozen work units bound to real entities
status: accepted
features:
- FT-117
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:acaa7b861f952f12c60e4adadf10dbbde3694bd212104e0ceedc2c3cbbf543d8
source-files:
- product-core/src/pf/dispatch.rs
- product-cli/src/commands/cell.rs
---

## Context

A task type (FT-113) is a *template*: its cells reference dual-read slots
(`domain:entity`) that bind to concrete values only at dispatch. A work unit
(FT-116) is the *concrete*, frozen SPMC manifest. The missing step is dispatch:
turning a task type plus slot bindings into the work units it produces, with
the slot references resolved to the bound entities and the context frozen — the
point at which the realisation layer actually touches the captured domain.

## Decision

Implement `pf::dispatch` and `product cell dispatch <task-type> --bind
slot=value …`:

1. **Validate bindings against the What graph.** Every binding must name a
   declared slot; every required slot must be bound; and any slot a cell
   references as `domain:<slot>` (an *entity-referenced* slot) must bind to a
   value that exists as a node in the captured What graph. A bad binding is a
   blocking violation and nothing is instantiated.
2. **Instantiate each cell into a work unit.** The cell's `derived_from` slot
   pointers are resolved to the bound values (`domain:entity` → `domain:Order`),
   the context is frozen and content-hashed, a single-purpose prompt and the
   produced artifact are filled in, and a rationale trace (`what` = the bound
   entity, `why` = the cell's applied patterns) is emitted. The result is a
   valid `WorkUnit` that itself passes `work-unit validate`.

Entity-checking keys off *which slots a cell actually derives from via
`domain:`*, not the slot's declared `kind` — a `kind: domain` slot like
`fields` carries field names, not an entity id, so only slots that flow into a
domain pointer are required to resolve to entities.

Work units are written to `.product/work-units/<id>.yaml` (or `--print`ed),
ids keyed by the primary bound entity so repeated dispatches don't collide.

## Rationale

- Dispatch is where the template becomes concrete against real domain
  entities; validating bindings against the What graph up front is what keeps a
  dispatch honest (you cannot realise against an entity that doesn't exist).
- Producing standard `WorkUnit` manifests (rather than a bespoke output) means
  the dispatched units are checkable by the existing `work-unit validate` — the
  loop closes: a dispatched unit is a conformant SPMC unit.
- Keying entity-resolution off `domain:` derivation (not slot kind) matches how
  the dual-read schema actually uses slots and avoids false rejections.

## Rejected alternatives

- **Require every `kind: domain` slot to bind to an entity.** Rejected: domain
  slots like `fields`/`validation` carry data, not entity ids; only slots a
  cell derives from via `domain:` must resolve to entities.
- **Run the SPMC prompt during dispatch (emit the actual artifact).** Out of
  scope: dispatch freezes and emits the manifests; executing the model to
  produce code is a separate concern.

## Test coverage

- TC-970 — dispatch instantiates frozen work units with resolved domain pointers.
- TC-971 — the dispatched work units pass `work-unit validate`.
- TC-972 — binding to a non-entity is rejected (nothing written).
- TC-973 — a missing required binding is rejected.
- TC-974 — `--print` emits to stdout without writing files.
