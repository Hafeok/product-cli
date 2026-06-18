---
id: ADR-061
title: A Decider's signature is derived from and validated against the event model
status: accepted
features:
- FT-121
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/decider.rs
- product-core/src/pf/decider_turtle.rs
- product-core/src/pf/rules_decider.rs
- product-cli/src/commands/decider.rs
---

## Context

§3.3 introduces the Decider — the executable form of an aggregate's behaviour, a
pair of pure functions `decide`/`evolve`. What makes it conformant is that its
**signature is derived from, and validated against, the event model**; only the
decision logic is authored. The event model already specifies every part of the
signature: the commands targeting the aggregate, the events those commands emit,
the events that change it, and its invariants. Nothing in the toolchain yet
derived that signature or checked an authored Decider for drift from it.

## Decision

Add a `pf::decider` slice with two operations and surface them as
`product decider`:

1. **Derive** (`derive_decider(graph, aggregate)`) — read the captured What
   graph and build the full signature: `handles` = commands whose `targets` is
   the aggregate, `emits` = the de-duplicated union of those commands' emitted
   events, `evolves_from` = events whose `changes` is the aggregate, `rejects` =
   invariants whose `applies_to` is the aggregate.
2. **Validate** (`validate_decider(decider, graph)`) — check an authored Decider
   against the model. The three §3.3 anti-drift rules are expressed as **SPARQL
   graph rules** (`rules_decider`) over the combined What + Decider projection
   (`decider_to_turtle`), consistent with the project's "conformance lives in
   the graph" decision (ADR-060-adjacent / [[project-graph-conformance]]):
   - **No foreign commands** — a handled command must target the aggregate.
   - **Command coverage** — every command targeting the aggregate is handled.
   - **Output-alphabet containment** — an emitted event must be sanctioned by a
     handled command.
   Plus a structural rule that the `decides_for` target is a real Entity.

A derived Decider is conformant by construction, so `derive` then `validate` is
the happy path; `validate` exists to catch an authored Decider that has drifted.

## Rationale

- Deriving the signature from the model — rather than asking the author to
  restate it — is the property that makes the Decider an honest oracle: it cannot
  claim to handle a command the model does not have.
- Expressing the drift rules as SPARQL over the projection keeps every
  cross-reference check in one place (the graph), matching how the How and What
  conformance rules are already run, and reuses the existing oxigraph runner.
- Keeping the decision *logic* and the before-realisation *simulation* out of
  this increment bounds it to a verifiable, mechanical capability; the logic
  needs a separate authoring design.

## Rejected alternatives

- **Author the whole Decider signature by hand and only lint it.** Rejected: the
  signature is fully determined by the event model; authoring it invites drift
  the framework explicitly forbids. Derivation makes the conformant case free.
- **Native field-walk for the drift rules.** Rejected: these are graph
  cross-references; the project standard is to express them as SPARQL rules, not
  hand-rolled walks that drift from the shapes.

## Test coverage

- TC-946 — derive a signature and validate it conformant; show / list.
- TC-947 — a Decider handling a foreign command is non-conformant.
- `pf::decider` + `pf::rules_decider` unit tests cover derivation and each drift
  rule (foreign command, missing coverage, unsanctioned event, non-entity
  aggregate).
