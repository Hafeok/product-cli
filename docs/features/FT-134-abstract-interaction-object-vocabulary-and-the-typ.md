---
id: FT-134
title: Abstract Interaction Object vocabulary and the typed UiStep
phase: 7
status: complete
depends-on:
- FT-110
adrs:
- ADR-078
tests:
- TC-994
- TC-995
- TC-996
domains:
- api
- data-model
domains-acknowledged:
  ADR-040: AIOs/UiStep are What-side artifacts at the What/How boundary; the structural rule composes with the existing What-side rules; the verify pipeline is untouched.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-048: Reads/writes the captured What graph only (the domain session); no other side effects.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code, stdout).
  ADR-043: AIO seeding, the UiStep model, and the AIO-only rule live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::rules_ui`/`pf::model` carry unit tests. No property or session dimension for a vocabulary + structural rule.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-041: Additive — seeds the AIO/ContextOfUse node kinds and the UiStep edges; WireframeStep is kept as a deprecated alias, so no removal/absence TC is required this increment.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
patterns:
- PAT-001
---

## Description

§3.2.1 and §3.2.2 of the framework specify the What of a screen as a **UI step**
whose interactions are **typed against Abstract Interaction Objects (AIOs)** —
`single-select`, `trigger-action`, `text-entry`, `display-value`,
`display-collection`, `navigate`, `edit`, … — never against a concrete control.
An AIO is *meaning*; a concrete control is *realisation*. Because the two are
distinct kinds of node, "a UI step naming a dropdown" is a structural violation,
not a style lapse.

This feature lays the foundation the rest of the UI system builds on (ADR-078):
the AIO vocabulary, the contexts of use, the typed `UiStep` node that supersedes
the `WireframeStep` stub, and the structural rule that enforces the AIO-only
boundary.

## Functional Specification

### Inputs

- The captured What graph for a product (the domain session; `--product` to
  override the default).
- AIO references and context-of-use declarations supplied when authoring a
  `UiStep` (the projection it surfaces, the commands it offers, its transitions,
  each interaction typed against an AIO).

### Behaviour

- **Seed the core AIO set.** The closed-core AIOs of the §3.2.2 table are
  recognised registry entries (`Aio` nodes): `trigger-action`, `single-select`,
  `multi-select`, `text-entry`, `numeric-entry`, `date-entry`, `display-value`,
  `display-collection`, `navigate`, `edit`. `product domain list aio`
  lists them. An adopter may register an additional AIO (a named,
  modality-independent interaction kind with a declared arity over domain data).
- **Declare contexts of use.** `ContextOfUse` nodes capture form factor (phone,
  tablet, desktop) and modality (pointer, touch, voice) as What-side facts that
  carry no realisation — the parameter later reification rules are written
  against (ADR-083).
- **Author a `UiStep`.** The node carries the §3.2.1 buildable core as typed
  edges: `surfaces` → the projection(s) shown (through display AIOs), `offers` →
  the commands valid at the step (through action/input AIOs), `transitions_to` →
  the next step on an action or event. Each referenced interaction is `typed_as`
  exactly one AIO. `intent` is the single permitted free-text field (the marked
  residue, §3.2.1). `WireframeStep` is accepted as a deprecated alias and
  migrated.
- **The structural AIO-only check** (`pf::rules_ui`). A SPARQL graph rule over
  the What projection: a UI step's interactions may reference only `Aio`-typed
  nodes; a reference to a CIO or any non-AIO control fails the check. Runs with
  the other What-side rules under `product graph check` / the framework's
  conformance path; exits non-zero on violation, naming the offending step and
  reference.

### Error handling

- Authoring a `UiStep` interaction `typed_as` an unknown AIO is a clear error
  pointing at the recognised set.
- The AIO-only rule reports each violating (step, reference) pair, not a bare
  fail.

## Out of scope

- **CIOs and reification** (`reify(AIO, context) → CIO`), the design system, and
  tokens are ADR-083/FT-139; this feature only names CIO as the forbidden
  reference target.
- **State meanings, accessibility obligations, and content references** on the
  Ui step are later increments (FT-136 / FT-137 / FT-138).
- **The page graph** semantics of `transitions_to` (flow subgraphs, application
  root, derived top-level) are FT-135; here a transition is a plain typed edge.
- **The seam verification** that consumes `surfaces`/`offers` to check the step
  against the projection and the Decider's commands is FT-140.

## Acceptance

- TC-994 — seed and list the core AIO set; declare a context of use.
- TC-995 — author a `UiStep` typed against core AIOs (surfaces a projection,
  offers a command); the structural AIO-only check passes.
- TC-996 — a `UiStep` referencing a non-AIO (CIO) control fails the AIO-only
  rule with a graph-conformance finding naming the step and reference.
