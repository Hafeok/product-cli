---
id: ADR-078
title: UI steps are typed against Abstract Interaction Objects; AIOs and CIOs are graph nodes
status: accepted
features:
- FT-134
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:08d46f72d2aec6abc42830a069386767cd07db3eacb97b3276a3e59ba25c1800
source-files:
- product-core/src/pf/ids.rs
- product-core/src/pf/model.rs
- product-core/src/pf/ops.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/rules_ui.rs
- product-cli/src/commands/domain_fields.rs
---

## Context

§3.2.1 and §3.2.2 of the framework specify the What of a screen as a **UI step**
whose interactions are **typed against Abstract Interaction Objects (AIOs)** — a
closed, extensible vocabulary of context-independent interaction kinds
(`single-select`, `trigger-action`, `text-entry`, `display-value`,
`display-collection`, `navigate`, `edit`, …) — never against a concrete control
from a design system. The framework is explicit that this is not a matter of
discipline but of **type**: "a UI step naming a dropdown" must be a structural
violation a verification rejects, the same way the layout allowlist rejects a
misplaced file, because AIOs and concrete controls are *distinct kinds of node
in the graph*.

The `pf/` engine today has only a stub: `WireframeStep` carries four free-text
fields (`id`, `label`, `triggers`, `displays`, `product-core/src/pf/model.rs`)
with no AIO vocabulary, no typed interaction edges, and no structural guard. The
richer UI step of §3.2.1 — its buildable core (information shown, actions
available, transitions) typed against AIOs — has no representation, so the
What/How UI split is currently advisory prose rather than a graph fact.

## Decision

Establish the AIO type boundary as graph structure, in three parts:

1. **New node kinds.** Add `Aio` and `ContextOfUse` to `NodeKind`
   (`pf/ids.rs`/`ALL_KINDS`) and the `pf:` ontology
   (`schema/ontology/product-framework.ttl`). Seed the **closed core** AIO set
   (§3.2.2 table) as recognised registry entries; an adopter may register
   additional AIOs against the same definition (a named, modality-independent
   kind of interaction with a declared arity over domain data). Declare the
   relevant **contexts of use** (form factor, modality, …) as `ContextOfUse`
   nodes — What-side facts that carry no realisation.

2. **Supersede `WireframeStep` → `UiStep`.** Evolve the node from four free-text
   fields into the §3.2.1 two-layer model. Its **buildable core** is expressed
   as typed edges, not strings: `surfaces` → the projection(s) it shows (through
   display AIOs), `offers` → the commands valid at the step (through action/input
   AIOs), `transitions_to` → the next step on an action or event. Each
   interaction the step references is `typed_as` exactly one AIO. `WireframeStep`
   remains a deprecated alias for one release, migrated by `ops`/`session`.

3. **The structural AIO-only check** (`rules_ui`). A SPARQL graph rule over the
   What projection: **a UI step's interactions may reference only `Aio`-typed
   nodes; a reference to a CIO (or any non-AIO control) fails.** This is the
   cheap by-construction gate that makes the What/How UI split structural rather
   than advisory — the type boundary §3.2.1 requires. It runs with the other
   What-side rules ([[project-graph-conformance]]).

CIOs are named here as the *forbidden* reference target so the boundary is
well-defined; the CIO node kind itself and its reification rules are deferred to
ADR-083, and the seam verification that consumes these edges to ADR-084.

## Rationale

- Making `Aio`/`Cio` distinct node kinds is what lets "a UI step naming a
  dropdown" be a graph-level type error caught by a rule, exactly as the
  framework demands — not a style lapse left to review.
- Expressing the buildable core as typed edges (`surfaces`/`offers`/
  `transitions_to`) rather than free text means the seam verification (ADR-084)
  can later check the step against the projection and the Decider's commands
  mechanically, the same way the Decider's signature is checked (ADR-061).
- Seeding the core AIO set as a closed registry keeps common cases
  interoperable across instances while leaving the set extensible, per §3.2.2.
- Reusing the SPARQL rule harness (`sparql_rules.rs` + `rules_what.rs`) keeps
  every cross-reference check in the graph, consistent with the project standard.

## Rejected alternatives

- **Keep `WireframeStep`'s free-text `triggers`/`displays` and lint the prose.**
  Rejected: free text cannot be type-checked, so the AIO/CIO boundary stays
  advisory — the precise fusion §2 forbids. The boundary must be structural.
- **Model AIOs as a string enum on the step rather than as nodes.** Rejected: an
  enum cannot be extended by an adopter, cannot carry inherited accessibility
  obligations (ADR-081), and cannot be the target of reification rules
  (ADR-083). AIOs must be first-class nodes.
- **Add a parallel `UiStep` kind beside `WireframeStep`.** Rejected: two UI-step
  kinds would split every downstream rule. Supersede with a deprecation alias.

## Test coverage

- TC — seed the core AIO set and list it; declare a context of use.
- TC — author a `UiStep` whose interactions are `typed_as` core AIOs; it
  surfaces a projection and offers a command; the structural check passes.
- TC — a `UiStep` referencing a non-AIO (CIO) control fails the structural
  AIO-only rule with a graph-conformance finding.
- `pf::rules_ui` unit tests cover the AIO-only rule (pass and each failure
  shape); `pf::model`/`pf::ops` unit tests cover the `WireframeStep`→`UiStep`
  migration and the typed-edge ops.
