---
id: FT-113
title: product cell — validate task-types against the What graph and How contract
phase: 6
status: complete
depends-on:
- FT-111
- FT-112
adrs:
- ADR-055
tests:
- TC-930
- TC-931
- TC-932
- TC-933
- TC-934
- TC-935
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `cell` subcommand family; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Followed — model + validation live in pure `pf::cell*` slices; the CLI is a thin BoxResult adapter.
  ADR-048: Reads a task-type YAML (default `.product/cell.yaml`), the What session, and the How contract; writes only on `init`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Six scenario TCs drive the binary through the assert_cmd harness; the pf::cell slices carry unit tests. No property or session dimension for a file validator.
  ADR-040: The task type is a structural realisation artifact; the checker cross-validates it against the What/How graphs; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product cell` is the CLI over the framework's §5 **realisation layer** — the
task-type definition (e.g. `add-crud-resource`): the dual-read `slots`, the
**cells** (SPMC work units) it dispatches, and the **audits** that back them.

The point of a cell is that it is built *from the rest of the graph*: its
frozen input is `derived_from` domain concepts (`domain:Order`) and it
`applies` patterns from the archetype's How contract. This feature makes that
checkable — `product cell validate` cross-checks a task type against the
captured What graph (FT-109/110) and the How contract (FT-111), so cells use
the entities and patterns that actually exist, not free-floating strings.
Design rationale is ADR-055.

## Functional Specification

### Inputs

- A task-type YAML file, default `.product/cell.yaml`, overridable with
  `--file`. `init` takes a task-type `<id>` and `--archetype`.
- Best-effort context for cross-validation: the default product's What graph
  (`.product/author-domain/<product>/`, `--product` to override) and the
  archetype's How contract (`.product/how-contract.yaml`).

### Behaviour

- `product cell validate` — structural check (≥1 slot, ≥1 audit, no slot
  without an inline audit, every audit names what it protects, every cell
  declares `derived_from`) plus cross-checks: each `domain:X` cell input
  resolves to a declared domain slot or a real node in the What graph; each
  `applies` names a How pattern/principle; bare/`slot:` inputs name a sibling
  cell or slot. Reports which graphs were cross-checked. Exits 1 on a blocking
  structural violation; cross-reference gaps are warnings.
- `product cell show` — a summary (id, archetype, classification, counts).
- `product cell list <slots|cells|audits>` — item rows.
- `product cell init <id> [--archetype N]` — scaffold; refuses to overwrite
  without `--force`.

### Error handling

- A blocking structural violation prints the framework-section message and
  exits 1; dangling domain/pattern pointers are warnings.
- A missing task-type file is a clear error pointing at `product cell init`.
- An unknown `list` kind exits non-zero with the accepted kinds.

## Out of scope

- It does not dispatch cells or run their SPMC prompts (realisation execution
  is future work); it validates and inspects the definition.
- It does not author the What or the How (FT-109/110/111 cover those).
- Standalone work-unit (`work-unit.yaml`) validation is a future increment;
  this covers the task-type container and its embedded cells.

## Acceptance

- TC-930 — validate passes on the conformant example.
- TC-931 — a slot with no inline audit is a blocking violation.
- TC-932 — a dangling `domain:` pointer is cross-checked against the What graph.
- TC-933 — `applies` is cross-checked against the How contract.
- TC-934 — show/list/init work.
- TC-935 — validate without a file is a clear error.
