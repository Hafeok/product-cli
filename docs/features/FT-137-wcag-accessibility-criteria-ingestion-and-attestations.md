---
id: FT-137
title: WCAG accessibility criteria ingestion and attestations
phase: 7
status: planned
depends-on:
- FT-134
adrs:
- ADR-081
tests:
- TC-1003
- TC-1004
- TC-1005
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds the WcagCriterion/Attestation node kinds and the must_satisfy/attests edges; nothing is removed or deprecated, so no absence TC is required this increment.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: WCAG ingestion, the inherited-union computation, the machine gate, and the attestation-coverage rule live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-048: Reads/writes the captured What graph only (the domain session plus ingested WCAG reference data); no other side effects.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code, stdout).
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::wcag`/`pf::rules_ui` carry unit tests. No property or session dimension for ingestion + a coverage rule.
  ADR-040: WCAG criteria are What-side artifacts at the What/How boundary; the gate and attestation-coverage rules compose with the existing What-side UI rules; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

§3.2.3 of the framework specifies a UI step's **accessibility obligations** as a
set of **WCAG 2.2 success criteria** referenced as ingested entities — not as
free-text prose. Each criterion is tagged by verification type (machine /
assisted / manual), most obligations are *inherited* from the AIOs a step uses,
and the verdict reports a conformance level and its basis rather than a bare
pass.

This feature ingests the WCAG 2.2 criterion hierarchy as graph entities, attaches
`must_satisfy` obligations to the seed AIOs and to UiSteps, computes a step's
obligation union, gates the machine criteria deterministically, and discharges
the assisted/manual ones through dated, attributed attestations (ADR-081).

## Functional Specification

### Inputs

- The captured What graph (the domain session) with seed AIOs (FT-134) and
  authored UiSteps.
- The ingested WCAG 2.2 reference data (principle → guideline →
  success-criterion → level, each criterion tagged machine / assisted / manual).
- Attestations supplied for assisted/manual criteria (each dated and attributed).

### Behaviour

- **Ingest WCAG 2.2.** The criterion hierarchy is available as `WcagCriterion`
  nodes carrying standard id, level (A/AA/AAA), and verification type. The seed
  AIO set declares the criteria each AIO `must_satisfy` (e.g. `text-entry` →
  labelling criteria; image-bearing `display-value` → 1.1.1 Non-text Content).
- **Attach and extend.** A `UiStep` adds screen-specific criteria via
  `must_satisfy`; its full obligation is the **computed union** of its AIOs'
  criteria plus its own. `product domain show <step>` surfaces the union and the
  inheritance source of each criterion.
- **Discharge by type.** Machine criteria are deterministic gates; an unsatisfied
  machine criterion fails. Assisted/manual criteria are discharged by an
  `attests` record (dated, attributed, immutable once recorded); an undischarged
  one fails.
- **Verdict.** The accessibility verdict reports the conformance **level and its
  basis** — which criteria were required, and for each whether it passed a
  machine gate or carries an attestation — never a bare pass.

### Error handling

- A `must_satisfy` referencing an unknown criterion is a clear error pointing at
  the ingested set.
- An attestation missing its date or attribution is rejected at the frozen
  boundary.

## Out of scope

- **The machine-check implementations** (the actual deterministic checks behind
  each machine criterion) are out of scope by the framework's open/closed line —
  the framework specifies the criterion and its verification type; the adopter
  supplies the check.
- **Composing the accessibility verdict into the full screen seam verdict**
  (alongside reification, state, and content coverage) is the seam verification
  (FT-140).

## Acceptance

- TC-1003 — a step inherits its AIOs' criteria as a computed union; adding/removing
  an AIO changes the union with no hand-maintained list.
- TC-1004 — an unsatisfied machine criterion fails the gate; the verdict reports
  level and basis, not a bare pass.
- TC-1005 — an assisted/manual criterion is undischarged without an attestation
  and discharged by a dated, attributed one.
