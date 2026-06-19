---
id: ADR-080
title: Read models declare a state space; UI steps cover it constrained and complete
status: accepted
features:
- FT-136
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:e95b3e393c023d40ebc6815f89e290dde9683ac2e37a152c22938790db218775
source-files:
- product-core/src/pf/model.rs
- product-core/src/pf/ops.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/rules_ui.rs
- product-core/src/pf/projector.rs
---

## Context

§3.2 of the framework says a read model is not always a present value: it
declares (or makes inferable) a **state space** — `present` plus any of
`loading`, `empty`, and `failed` it can exhibit. §3.2.1 ("Projection states as
meaning") then makes the screen responsible for that space: what each state
*means to the user* is a behavioural fact, not styling, and a UI step's state
annotations must be both **constrained** (it may annotate only states the
projection actually has — no "empty" meaning for a projection that cannot be
empty) and **covering** (it must give a meaning to *every* state the projection
can be in, or explicitly waive one with a reason).

The dangerous case this exists to catch is the **forgotten state**: a projection
that can `fail` whose screen never says what failure means. Today the `pf/`
engine has no state space on `ReadModel` (`product-core/src/pf/model.rs`) and no
rule tying a UI step's annotations to it, so the forgotten-state defect is
invisible to the toolchain — exactly the under-specification the framework's
coverage discipline is meant to expose.

This is the UI analogue of the Decider's **command-coverage** rule (§3.3,
ADR-061): exhaustiveness over an alphabet. Where the Decider must handle every
command targeting its aggregate, a UI step must mean every state its surfaced
projection can occupy.

## Decision

Add the state space to read models and a constrained-and-covering check over UI
steps, expressed in the graph:

1. **State space on `ReadModel`.** A read model declares its state space —
   `present` plus any of `loading`/`empty`/`failed` it can exhibit — as a field
   on the node (`pf/model.rs`/`ops.rs`), projected to turtle alongside its
   existing `projects` edges. A Projector (§3.4) makes the space inferable where
   the fold determines it; the declared space is the alphabet the check ranges
   over.

2. **State annotations on `UiStep`.** A UI step that `surfaces` a projection
   annotates, per surfaced-projection state, *what that state means to the user*
   — or records an explicit **waiver** carrying a written reason. For `failed`,
   the meaning is What ("the user must know it can't be shown and how to
   recover"); the failure *mechanism* is How — the same split as an accessibility
   criterion versus its discharge (§3.2.3).

3. **The coverage rule** (`pf::rules_ui`, modelled on `pf::rules_decider`). A
   SPARQL graph rule over the What projection enforcing both halves:
   - **Constrained** — a step may annotate only states the surfaced projection
     declares; annotating a state the projection cannot exhibit fails.
   - **Covering** — every state the projection can be in is either given a
     meaning by the step or explicitly waived with a reason; a missing, unwaived
     state fails, naming the forgotten state.

The **waiver** is the escape hatch the Decider's command coverage does not get,
because some states are legitimately ignorable (a load too fast to perceive); a
waiver without a reason is itself a violation.

## Rationale

- Making the state space a declared alphabet and the annotations a covering map
  over it turns "did we handle the error screen?" from a review question into a
  graph rule — the forgotten-state bug becomes a deterministic failure, not a
  production surprise.
- Reusing the `pf::rules_decider` shape keeps every coverage check the same kind
  of artifact (SPARQL over the projection), consistent with the project standard
  that conformance lives in the graph ([[project-graph-conformance]]).
- The waiver keeps the rule honest without making it tyrannical: ignorable
  states are dismissable, but only on the record and with a stated why, so the
  dismissal is itself reviewable.
- Splitting `failed`'s *meaning* (What) from its *mechanism* (How) keeps the UI
  step a pure statement of meaning, leaving the recovery implementation to the
  screen-composition contract — the funnel principle applied to error states.

## Rejected alternatives

- **Free-text "what happens on error" prose on the step.** Rejected:
  unverifiable — a step can claim to handle errors while the surfaced projection
  has a `failed` state no annotation covers, so the forgotten-state defect
  survives. Coverage must range over a declared alphabet.
- **No waiver — force a meaning for every state.** Rejected: it would compel
  meaningless annotations on legitimately ignorable states (an imperceptible
  load), training authors to write noise. The waiver-with-reason preserves the
  signal.
- **Infer the state space silently from the Projector only, never declared.**
  Rejected: a trivial CRUD view has no Projector yet still has a state space;
  the space must be declarable directly, with Projector inference as an
  additional source, not the only one.

## Test coverage

- TC-1000 — a UI step that means every state of a {present, empty, failed}
  projection passes the coverage check.
- TC-1001 — the same step omitting the `failed` meaning, unwaived, fails the
  rule, naming the forgotten state.
- TC-1002 — waiving an ignorable `loading` state with a reason passes;
  annotating a state the projection cannot exhibit fails the constrained half.
- `pf::rules_ui` unit tests cover constrained and covering independently (each
  pass and failure shape); `pf::model`/`pf::ops` unit tests cover the state-space
  field and the annotation/waiver ops.
