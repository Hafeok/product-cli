---
id: FT-136
title: Read-model state space and UI-step state coverage
phase: 7
status: complete
depends-on:
- FT-134
adrs:
- ADR-080
tests:
- TC-1000
- TC-1001
- TC-1002
domains:
- api
- data-model
domains-acknowledged:
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::rules_ui`/`pf::model` carry unit tests. No property or session dimension for a coverage rule.
  ADR-041: Additive — adds a state-space field to read models and state-meaning/waiver annotations to UiSteps; nothing is removed or deprecated, so no absence TC is required.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-048: Reads/writes the captured What graph only (the domain session); no other side effects.
  ADR-040: The state space and its coverage rule are What-side artifacts at the What/How boundary; they compose with the existing What-side rules; the verify pipeline is untouched.
  ADR-051: Every TC declares `observes:` (graph, exit-code) and asserts on those surfaces.
  ADR-043: The state space, the annotations, and the coverage rule live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
patterns:
- PAT-001
---

## Description

§3.2 of the framework gives a read model a **state space** — `present` plus any
of `loading`/`empty`/`failed` it can exhibit — and §3.2.1 makes the screen mean
that space: a UI step's state annotations must be **constrained** (only states
the projection has) and **covering** (every state given a meaning, or explicitly
waived with a reason). This is the UI analogue of the Decider's command-coverage
rule (§3.3), and the defect it catches is the *forgotten state* — a projection
that can fail whose screen never says what failure means.

This feature adds the state space to read models, the per-state meaning and
waiver annotations to `UiStep`, and the constrained-and-covering check (ADR-080).

## Functional Specification

### Inputs

- The captured What graph (the domain session; `--product` to override).
- A read model's declared state space (`present` plus any of
  `loading`/`empty`/`failed`).
- A `UiStep`'s per-state meaning annotations, or a waiver carrying a reason, for
  each surfaced-projection state.

### Behaviour

- **Declare a state space.** A read model carries its state space as a field;
  `product domain show <read-model>` surfaces it. Where a Projector (§3.4)
  determines the space, it is inferable; the declared space is the alphabet the
  coverage check ranges over.
- **Annotate a UiStep.** For each state of a surfaced projection, the step
  records either a meaning (what the state means to the user) or a waiver with a
  written reason. For `failed`, the meaning is What ("the user must know it can't
  be shown and how to recover"); the failure mechanism is left to the How.
- **The coverage check** (`pf::rules_ui`, run under `product graph check` / the
  framework's What-side conformance path):
  - **Constrained** — a step may annotate only states the surfaced projection
    declares; annotating an impossible state fails, naming it.
  - **Covering** — every state the projection can be in is meant or waived; a
    missing, unwaived state fails, naming the forgotten state. Exits non-zero on
    any violation.

### Error handling

- Annotating a state the projection does not declare is a clear
  constrained-violation error naming the (step, state) pair.
- A surfaced projection state with neither a meaning nor a waiver is a clear
  covering-violation error naming the forgotten state.
- A waiver without a reason is itself a violation.

## Out of scope

- **The full seam verification** that composes state coverage with reification,
  content, and accessibility discharge into one verdict is FT-140; this feature
  ships state coverage as a standalone What-side rule.
- **The How-side failure mechanism** (how a `failed` state is actually rendered
  and recovered) is the screen-composition contract (FT-139), not this feature.
- **Projector-driven inference** of the state space beyond the declared field is
  bounded to what the existing Projector already determines.

## Acceptance

- TC-1000 — a UI step that means every state of a {present, empty, failed}
  projection passes the coverage check.
- TC-1001 — the same step omitting the `failed` meaning, unwaived, fails the
  rule, naming the forgotten state.
- TC-1002 — waiving an ignorable `loading` state with a reason passes;
  annotating a state the projection cannot exhibit fails the constrained half.
