---
id: ADR-079
title: Navigation is one page graph with named flow subgraphs and a declared application root
status: accepted
features:
- FT-135
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:5c861d656c75c5a3c54f7604ee0a6f57f795ec59d97e75177ba2095775a2bb11
source-files:
- product-core/src/pf/ids.rs
- product-core/src/pf/model.rs
- product-core/src/pf/ops.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/query.rs
- product-core/src/pf/rules_ui.rs
---

## Context

§3.2.4 of the framework states that navigation is **not a separate "shell"**
layered above screens: it is **one graph whose nodes are pages (UI steps) and
whose edges are `navigate` transitions**. A page's `transitions_to` edges
(ADR-078) *are* the edges of this graph; every way a user moves through the
application — within a task or across tasks — is an edge of the same kind. There
is no second navigation model to maintain.

The `pf/` engine today models a `Flow` as a flat ordered list of step ids
(`product-core/src/pf/model.rs:125`) with no navigate edges, no entry page, and
no notion of an application root. So navigation cannot be queried as a graph,
"top-level" has nowhere to come from, and the chrome a renderer draws has no
source in the model. This ADR makes the page graph a first-class graph fact.

## Decision

Model navigation as one page graph, in three parts:

1. **A flow is a named connected subgraph with a declared entry page.** Add the
   `in_flow` edge (a page belongs to a named flow) and an entry-page marker on
   `Flow`. Flows do **not own** navigation — they partition one shared graph, so
   two flows can share pages and link to each other (the existing ordered-list
   `Flow` gains entry-page + subgraph semantics; the order list is retired in
   favour of `transitions_to` edges). Just as a feature is a subgraph of the
   event model (§7), a flow is a named region of the page graph.

2. **The application root is a declared singleton node.** Add an
   `ApplicationRoot` node kind: the place a user is before entering any flow. Its
   out-edges — `navigates_from_root` — are the **global destinations** (the flows
   reachable from anywhere, which a renderer draws as primary navigation) and the
   global actions (a `trigger-action` valid app-wide). A `navigate` AIO *at the
   root* is exactly the burger menu / tab bar / sidebar.

3. **"Top-level" is derived, not a category.** A page is top-level iff it has an
   inbound `navigates_from_root` edge; a page is nested iff it is reachable only
   from another page. Nothing is tagged by hand. The application's primary
   navigation is therefore **computed** — the set of the root's out-edges — and
   it changes automatically as flows are added to or removed from the root. These
   are graph queries in `pf::query`; impact analysis ("what can reach this
   page?") is likewise a query ([[project-graph-conformance]]).

This reuses the `navigate` AIO (ADR-078) at graph scope — no new primitive, just
interaction one level up.

## Rationale

- One graph with named subgraphs keeps a single source of truth: the same
  `navigate` edge, the same reification (ADR-083), the same impact analysis.
  Treating chrome as the reified root-navigation of the very graph the screen
  lives in means there is no separate content to author and no second place for
  navigation to drift.
- Deriving "top-level" from the root's edges — rather than a hand-applied tag —
  mirrors how a feature's domain footprint falls out of its flow slice (§7), and
  removes a whole class of drift between a tag and the edges it should reflect.
- A declared entry page per flow gives the seam verification (ADR-084) and any
  renderer a well-defined place to begin a flow, without an ordering convention.

## Rejected alternatives

- **A separate application-shell model sitting above flows.** Rejected: it would
  duplicate the transition machinery at a second level and create two places
  navigation can drift. The chrome is the reified root-navigation of the one
  page graph, not a parallel artifact.
- **Tag pages "top-level" by hand.** Rejected: top-level is fully determined by
  the root's out-edges; a hand tag invites exactly the tag-vs-edges drift the
  framework forbids. It must be derived.
- **Keep `Flow` as an ordered step list.** Rejected: an ordered list cannot
  express branching, shared pages, or cross-flow links; navigation is a graph,
  not a sequence.

## Test coverage

- TC-997 — mark a flow's entry page and navigate edges; the graph records
  `in_flow` and `transitions_to` forming a connected subgraph.
- TC-998 — "top-level" is derived: a page with an inbound root edge is top-level
  and in primary navigation; a page reachable only from another is nested.
- TC-999 — primary navigation recomputes automatically when a flow's entry page
  gains a `navigates_from_root` edge.
- `pf::query` / `pf::rules_ui` unit tests cover the derived top-level query, the
  computed primary-navigation set, and entry-page / subgraph structure.
