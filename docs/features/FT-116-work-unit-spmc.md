---
id: FT-116
title: product work-unit — validate an SPMC work unit against the What graph and How
phase: 6
status: complete
depends-on:
- FT-111
- FT-113
adrs:
- ADR-058
tests:
- TC-960
- TC-961
- TC-962
- TC-963
- TC-964
- TC-965
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `work-unit` subcommand family; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Followed — model + validation live in pure `pf::work_unit*` slices; the CLI is a thin BoxResult adapter.
  ADR-048: Reads a work-unit YAML (default `.product/work-unit.yaml`) + the What session + the How contract; writes only on `init`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Five scenario TCs drive the binary through the assert_cmd harness; the pf::work_unit slices carry unit tests. No property or session dimension for a file validator.
  ADR-040: The work unit is a structural realisation manifest; the checker cross-validates it against the What/How graphs; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product work-unit` is the CLI over the framework's §5 **work unit** — the
smallest reproducible unit of realisation (SPMC: Schema, Prompt, Model,
Context), producing one artifact and carrying a rationale trace. Where a cell
inside a task type is the template, a `work-unit.yaml` is the concrete,
dispatchable manifest whose frozen input names real domain concepts. This
feature validates it against the captured What graph (FT-109/110) and the How
contract (FT-111).

## Functional Specification

### Inputs

- A work-unit YAML file, default `.product/work-unit.yaml`, overridable with
  `--file`.
- Best-effort context: the default product's What graph (`--product` to
  override) and the How contract. A dispatched unit under
  `.product/archetypes/<name>/work-units/` cross-checks against that
  archetype's `how-contract.yaml`; otherwise `.product/how-contract.yaml`.

### Behaviour

- `product work-unit validate` — structural check (non-empty schema + prompt, a
  frozen context, a non-empty `context.derived_from`, one produced artifact)
  plus cross-checks: each `domain:X` input and `trace.what` resolve to a real
  node in the What graph; `applies`/`trace.why` name real How
  patterns/principles; an applied-but-unenforced principle surfaces a
  trace-truth warning. Reports which graphs were cross-checked. Exits 1 on a
  blocking structural violation; cross-reference gaps are warnings.
- `product work-unit show` — a summary (id, artifact, model, frozen context
  inputs, applied patterns).
- `product work-unit init <id>` — scaffold a starter manifest; refuses to
  overwrite without `--force`.

### Error handling

- An unfrozen context, empty `derived_from`, or empty artifact is a blocking
  violation.
- Dangling `domain:`/`applies` pointers are warnings.
- A missing work-unit file is a clear error pointing at `product work-unit init`.

## Out of scope

- It does not execute the SPMC prompt (run the model to emit the artifact);
  that is realisation execution, future work.
- It does not generate work units from a task type — that is `product cell
  dispatch` (FT-117).

## Acceptance

- TC-960 — validate passes on the bundled example.
- TC-961 — an unfrozen context is a blocking violation.
- TC-962 — a `domain:` input is cross-checked against the What graph.
- TC-963 — show / init work.
- TC-964 — validate without a file is a clear error.
