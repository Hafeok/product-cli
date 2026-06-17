---
id: ADR-065
title: A delivery slice is a pointer into the event model; its context is derived from the graph
status: accepted
features:
- FT-124
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/slice.rs
- product-core/src/pf/bundle.rs
- product-cli/src/commands/slice.rs
---

## Context

§7.1 says a delivery feature is a subgraph of the What — "not a free-floating
ticket." The point of building on a connected graph (§2) is that "describe this
system" is a query, not a stale document. A delivery unit should therefore carry
no restated behaviour: it should *point* at the model and let the concrete
build-context be assembled on demand. The toolchain already had a What-graph
bundle assembler (`pf::bundle::bundle`) for a single ad-hoc focus node, but no
saved, named delivery unit and no multi-anchor closure.

## Decision

Add a `Slice` artifact (`pf::slice`) and `product slice`:

- A slice is a pure pointer — `{ id, anchors: [node-id…], depth? }` — stored at
  `.product/slices/<id>.yaml`. It restates nothing about the behaviour.
- `validate_slice` requires at least one anchor and that every anchor resolves
  to a real node in the captured What graph; `slice new` refuses to save an
  unresolved pointer.
- The concrete context is **derived**, not stored: `bundle::bundle_many`
  generalises the single-focus bundle to the union of the subgraphs reachable
  from all anchors, rendered as the same LLM-ready markdown bundle. `bundle`
  becomes a one-anchor call into it.

The command is named `slice` (not `feature`) because `product feature` already
manages the FT-XXX specification graph; this is the §7.1 delivery slice over the
What.

## Rationale

- A pointer + derived closure is the graph paying off: the feature cannot drift
  from the model because it contains none of the model — it names a location and
  the context is recomputed from the live graph each time.
- Reusing the existing bundle closure (rather than a bespoke slice extractor)
  keeps one definition of "the neighbourhood of a node" and inherits its tested
  depth/adjacency behaviour; generalising to a set of anchors is the only new
  capability, and `bundle` delegating to `bundle_many` keeps a single code path.
- Validating anchors at `new` time turns "a pointer to nothing" into an
  immediate error instead of an empty bundle later.

## Rejected alternatives

- **Restate the slice's concepts in the feature file.** Rejected by §7.1/§2: a
  delivery unit is a subgraph, and restating invites drift — the whole reason
  the What is a graph.
- **A separate slice-specific extractor.** Rejected: it would duplicate the
  bundle's adjacency/depth logic; generalising the existing assembler is leaner.
- **Reuse `product feature`.** Rejected: that name owns the FT-XXX specification
  graph; conflating the two concepts would confuse both.

## Test coverage

- TC-958 — `slice context` assembles the reachable subgraph (commands, events,
  contexts) from a single anchor, restating nothing.
- TC-959 — `slice new` rejects a dangling anchor.
- `pf::slice` units cover validation (empty/dangling/resolving) and closure
  assembly (flow closure, multi-anchor union); `pf::bundle` units still cover the
  single-focus depth behaviour.
