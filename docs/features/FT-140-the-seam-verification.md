---
id: FT-140
title: The seam verification — screen against UI step
phase: 7
status: planned
depends-on:
- FT-134
- FT-135
- FT-136
- FT-137
- FT-138
- FT-139
adrs:
- ADR-084
tests:
- TC-1012
- TC-1013
- TC-1014
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — implements the already-declared `pf:seam` VerificationKind; nothing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) governs the `pf::seam` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The composite seam check lives in the pure `pf::seam` slice, composing the per-phase rules; the CLI is a thin adapter.
  ADR-048: Reads the captured What graph plus the How's reification/content declarations; emits a verdict, writing no graph state.
  ADR-051: Every TC declares `observes:` (graph, exit-code) and asserts on those surfaces.
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::seam` carries unit tests over the composition. No property or session dimension for a verdict.
  ADR-040: The seam is a What→How verification kind; it joins the required kinds of §6.3 and reports a conformance level and its basis, never a bare pass.
patterns:
- PAT-001
---

## Description

§4.5 ("The seam is verified") and §6.3 require the **seam verification** — the
check that a screen and the UI step it realises agree. A screen sits on the
What→How seam exactly as application sits on application↔runtime (§4.2). This
feature implements the `pf:seam` VerificationKind already declared in the
ontology, as the **integrator** that composes the per-phase checks the earlier
UI-system features each define.

It therefore **depends on FT-134..FT-139 being in place**: it reuses their data
and rules (AIO typing and the `surfaces`/`offers` edges from FT-134, state
coverage from FT-136, accessibility discharge from FT-137, content coverage from
FT-138, reification coverage from FT-139) rather than re-deriving them.

## Functional Specification

### Inputs

- The captured What graph for a product (the UI steps, projections, commands,
  AIOs, states, content references, and accessibility obligations).
- The How's declarations the sub-checks consult: the reification rules and the
  content-store/design-system manifests (FT-141/FT-142).
- A screen (`UiStep`) id or a flow id to verify (`--product` to override the
  default).

### Behaviour

- `product seam <step|flow>` runs the composite verification, in order:
  1. the cheap structural **AIO-only** gate (FT-134) runs first;
  2. **datum projected** — every displayed datum is `project`ed by a read model
     in the flow;
  3. **control maps to command** — every control maps to a command the step
     `offers` and the Decider `handles`;
  4. **reification coverage** — every referenced AIO reifies to a CIO for each
     declared context (FT-139);
  5. **state coverage** — every surfaced-projection state is given a meaning or
     explicitly waived (FT-136);
  6. **content coverage** — every content key resolves for each declared locale
     (FT-138);
  7. **accessibility discharged** — machine criteria pass as gates; assisted/
     manual criteria have attestations (FT-137).
- Emits a **verdict reporting the conformance level and its basis**, never a bare
  pass. Exits non-zero on any failing sub-check, **listing every** failure (not a
  single opaque fail).

### Error handling

- A screen with no flow, or a flow with no read model, is a clear error pointing
  at the missing What-graph element.
- A sub-check whose How input is absent (no reification manifest, no content
  store) reports that gap as the failing basis, not a crash.

## Out of scope

- The **individual sub-rule implementations** are owned by FT-134..FT-139; this
  feature composes them, it does not re-implement them.
- The **§11/§12 manifest validators** (FT-141 design-system, FT-142 content
  store) are separate; the reification and content coverages here *consult* those
  manifests but do not validate their internal wholeness.

## Acceptance

- TC-1012 — a fully-agreeing screen passes; the verdict reports level + basis.
- TC-1013 — an unprojected datum or a foreign command fails the seam, named.
- TC-1014 — reification, state, and content coverage gaps each fail the seam and
  are listed independently.
