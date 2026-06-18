---
id: FT-114
title: product archetype — assemble and validate How, layout, and cells as one
phase: 6
status: complete
depends-on:
- FT-111
- FT-113
adrs:
- ADR-056
tests:
- TC-940
- TC-941
- TC-942
- TC-943
- TC-944
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `archetype` subcommand family (plus a §4.3 layout model); nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Followed — assembly + validation live in pure `pf::archetype`/`pf::layout` slices; the CLI is a thin BoxResult adapter.
  ADR-048: Reads an archetype under `.product/archetypes/<name>/` + the What session; writes only on `init`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Five scenario TCs drive the binary through the assert_cmd harness; the pf::archetype/pf::layout slices carry unit tests. No property or session dimension for a file assembler.
  ADR-040: The archetype is a structural realisation aggregate; the checker composes the What/How/cell/layout shapes; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

An **archetype** is a reusable, pre-filled How for a recurring system shape;
a product realises one or more. `product archetype` assembles the three parts
of an archetype — its How contract (FT-111), its §4.3 repository layout model,
and its task-type cells (FT-113) — from `.product/archetypes/<name>/` and
validates the whole assembly in one command.

This is the keystone that ties the framework layers together: the What
(FT-109/110) the cells are built from, the How (patterns/contracts) they
apply, the layout that places their artifacts, and the cells themselves —
checked for internal conformance *and* cross-part coherence, with every
finding attributed to its part.

This feature also introduces the §4.3 **layout model** (`pf::layout`), the
archetype's third part: glob rules with allowlist semantics and the two
normative guards (every rule cites what it enforces; prohibitions carry
rationale).

## Functional Specification

### Inputs

- An archetype name, resolving to `.product/archetypes/<name>/` containing
  `how-contract.yaml`, `layout.yaml`, and `cells/*.yaml`.
- The default product's What graph (`--product` to override) for cross-checking
  cells' `domain:` inputs.

### Behaviour

- `product archetype validate <name>` — validate each part against its shapes
  (How, layout, every cell) and the cross-part coherence (a cell's archetype
  matches; the How's `layout_model` resolves; a layout's archetype matches).
  Findings are tagged with their part (`how/…`, `layout/…`, `<cell>/…`). Exits
  1 on any blocking violation; cross-part gaps and dangling pointers are
  warnings.
- `product archetype show <name>` — a summary (How contract id + counts, layout
  rule count, each cell).
- `product archetype list` — the archetypes under `.product/archetypes/`.
- `product archetype init <name>` — scaffold the directory with a How, a
  layout, and an example cell; refuses to overwrite without `--force`.

### Error handling

- An archetype with no How contract is a blocking violation (exit 1).
- A blocking violation in any part exits 1 with the part-tagged message.
- A missing archetype directory is a clear error pointing at
  `product archetype init`.

## Out of scope

- It does not dispatch cells or run SPMC prompts (realisation execution is
  future work).
- It does not enforce the layout against an actual repository tree (the layout
  checker validates the model's well-formedness, not a filesystem); applying
  layout rules to files is FT-120 (`product archetype check`).
- Standalone work-unit (SPMC) validation remains a separate increment.

## Acceptance

- TC-940 — validate a full assembled archetype.
- TC-941 — a part violation is reported tagged with its part.
- TC-942 — an archetype with no How contract is non-conformant.
- TC-943 — show / list / init work.
- TC-944 — cells' `domain:` inputs are cross-checked against the What graph.
