---
id: ADR-067
title: Delivery units are id-pointers — release to deliverable to slice
status: accepted
features:
- FT-126
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/deliverable.rs
- product-core/src/pf/release.rs
- product-cli/src/commands/deliverable.rs
- product-cli/src/commands/release.rs
---

## Context

§7.1 defines features and releases as subgraphs of the What — "not free-floating
tickets." FT-124 made the slice a pointer into the event model. The delivery
layer above it (the §7.1 feature, and the release that groups features) should
follow the same discipline: pointers by id, restating nothing, validated to
resolve. The command name `feature` is unavailable (the legacy FT-XXX graph owns
`product feature`).

## Decision

Add two pure artifacts and their thin adapters:

- **`Deliverable`** (`{ id, slice, acceptance[] }`) — the §7.1 delivery feature,
  pointing at exactly one slice plus its acceptance criteria. Surfaced as
  `product deliverable` (named to avoid the legacy `product feature` collision).
- **`Release`** (`{ id, features[] }`) — a set of deliverable ids that ship
  together. Surfaced as `product release`; members are given with `--feature`.

`validate_deliverable` checks the slice resolves against the set of saved slice
ids; `validate_release` checks every member resolves against the set of saved
deliverable ids. Both are pure functions taking the known-id set; the CLI
adapters gather those sets from `.product/{slices,deliverables}` and refuse to
write an artifact with a dangling reference. `product status` counts both in its
Delivery line.

The hierarchy is **release → deliverable → slice → event-model section**, each
referencing the next by id.

## Rationale

- Id-pointers (not embedded copies) keep the delivery layer a true view over the
  graph: a release is its members, a deliverable is its slice — change the slice
  and the deliverable follows, with no restated content to drift.
- Validating references at `new` time turns a typo into an immediate error rather
  than a broken rollup later.
- Pure validation over a known-id set keeps the core testable without the
  filesystem; the adapter is the only place that reads directories.

## Rejected alternatives

- **Embed the slice/feature content in the deliverable/release.** Rejected by
  §7.1 — delivery units are subgraphs, and copying invites drift.
- **Reuse `product feature`.** Rejected: it owns the legacy FT-XXX graph;
  `deliverable` keeps the two concepts distinct.
- **Compute `done`/closed now.** Deferred: the §7.2 predicates need What-graph
  verification status and directed-dependency closure (not yet built); shipping
  the hierarchy first keeps this increment honest.

## Test coverage

- TC-967 — the slice → deliverable → release chain is created and shown in
  `product status`.
- TC-968 — dangling slice/deliverable references are rejected.
- `pf::deliverable` / `pf::release` units cover round-trip + each validation
  (resolving, empty, dangling).
