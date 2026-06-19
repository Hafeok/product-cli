---
id: ADR-088
title: Framework-graph onboarding is a derived guide plus a signposted, seedable init
status: accepted
features:
- FT-143
- FT-144
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:5c57a175bca8ce6bacd0cc589262f5366e76168f4a152de28b9b71afd311d921
source-files:
- product-core/src/guide/mod.rs
- product-core/src/guide/plan.rs
- product-core/src/demo.rs
- product-cli/src/commands/guide.rs
- product-cli/src/commands/init.rs
- product-mcp/src/read_handlers.rs
---

## Context

The framework graph (What/How/Delivery) had every lifecycle command but no
on-ramp: `product init` scaffolded only the meta graph and pointed users at
`product feature new`, the framework commands stood alone with no "what next"
between them, and the entry point (`product author domain`) was undiscoverable.
A workshop participant on day one had no path from an empty directory to a
captured What.

## Decision

Onboarding is delivered as a **derived guide over framework-graph state**, plus a
**signposted, optionally pre-seeded init** — not as a new persistent model:

1. **`product guide`** (and the `product_guide` MCP tool) probe a pure
   `FrameworkState` snapshot from disk and compute the current journey stage as
   the first unmet step (CaptureWhat → FixWhat → AuthorHow → CarveSlice →
   WrapDeliverable → BuildIt), rendering a checklist, the stage's meaning, and the
   exact next command — naming real ids (a command for `slice --anchor`, a slice
   for `deliverable --slice`) when known. The guide stores nothing; it is a
   read-only derived view, consistent with "the graph is derived" (ADR-003).
2. **`product init`** prints a framework-aware Next-steps block pointing at
   `product guide` / `product author domain`, and **`--demo`** seeds a small
   conformant bookstore What model via the validated `pf::edit::create` path,
   under the canonical `.product/` layout (ADR-048).

The guide is the single source of "where am I / what next", shared verbatim by
CLI and MCP (CLI↔MCP parity, FT-118).

## Rationale

- A *derived* guide cannot drift from the graph: it reads the same state the
  commands act on, so its advice is always current — the same reason the graph
  itself is derived (ADR-003).
- Computing the stage as the first unmet step makes guidance unambiguous (one
  next move) and self-updating as the model grows.
- Seeding a real, conformant example via the production authoring path (not a
  hand-written fixture) guarantees the demo matches what users will produce.

## Rejected alternatives

- **A separate onboarding/wizard model persisted on disk.** Rejected: it would
  duplicate state the graph already holds and could drift; a derived view cannot.
- **Static "getting started" docs only, no command.** Rejected: docs can't tell a
  user where *their* graph is; the value is the state-aware next step.
- **A hand-authored demo fixture.** Rejected: it would drift from the real
  authoring rules; seeding through `pf::edit::create` keeps it honest.

## Test coverage

- TC-1019 — `product guide` on a fresh repo reports CaptureWhat with an unticked
  checklist and the next command; JSON carries the structured stage.
- TC-1020 — `product init --demo` seeds a graph that passes `product domain
  validate`, and `product guide` then shows the What captured and conformant.
- `product_core::guide` unit tests cover every stage and the real-id naming;
  `product_core::demo` reloads the seed and asserts conformance.
