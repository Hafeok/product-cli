---
id: FT-111
title: product how — validate, show, and project an archetype's How contract
phase: 6
status: complete
depends-on:
- FT-109
adrs:
- ADR-054
tests:
- TC-910
- TC-911
- TC-912
- TC-913
- TC-914
- TC-915
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `how` subcommand family; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: No context bundle or template change; the command does not render bundles.
  ADR-043: Followed — model/validation/projection live in the pure `pf::how*` slices; the CLI is a thin BoxResult adapter.
  ADR-048: Operates on a How-contract YAML file (default `.product/how-contract.yaml`); the FT/ADR/TC graph is untouched.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and its body asserts on those named surfaces.
  ADR-018: Six scenario TCs drive the binary through the assert_cmd harness; the pf::how slices carry unit tests. No property or session dimension for a file validator.
  ADR-040: The How is a structural artifact; the checker mirrors the framework SHACL (incl. the crown trace-truth rule); the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product how` is the CLI over an archetype's **How** — the §4 architecture
layer: the Why cascade (top decisions → principles → patterns), the
application + infrastructure contracts, the repository layout reference, and
interface contracts. Unlike the What (an RDF graph captured via an MCP
session, FT-109/110), the How is authored as a **YAML file** and projected
into the graph; this feature validates, renders, and projects it.

The centrepiece is the **crown rule** — *the rationale trace must be true*:
every principle a work unit applies must be enforced by a verification. The
checker mirrors `schema/shapes/how.shacl.ttl` (including that SPARQL rule)
natively, and the Turtle projection is cross-verified against the real shapes
with `pyshacl`. Design rationale is ADR-054.

## Functional Specification

### Inputs

- A verb: `validate`, `show`, `list <kind>`, `export`, or `init`.
- A How-contract YAML file, default `.product/how-contract.yaml`, overridable
  with `--file`. `init` also takes `--archetype` (defaults to the repo name).

### Behaviour

- `product how validate` — structural + conformance check: decision rationale,
  pattern-realizes, infra-conformsTo-app, interface-derivedFrom, principle
  earn-their-place, and the crown trace-truth rule. Prints warnings; exits 1
  if any blocking violation.
- `product how show` — a summary (archetype, contracts, counts).
- `product how list <decisions|principles|patterns|interfaces>` — item rows.
- `product how export` — project the contract to Turtle (the §4 nodes plus the
  synthesised Verification/Work-Unit links the cross-node shapes need).
- `product how init [--archetype N]` — scaffold a starter contract; refuses to
  overwrite without `--force`.

### Error handling

- A blocking conformance violation prints the framework-section message and
  exits 1; a dangling soft pointer (`licenses`, `realized_by`) is a warning.
- A missing contract file is a clear error pointing at `product how init`.
- An unknown `list` kind exits non-zero with the accepted kinds.

## Out of scope

- It does not author the contract through an agent (the How ships no
  `tools.json`); the file is edited directly, then validated/projected.
- It does not author the What (that is FT-109/110).
- It does not implement the interface-contract industry standards (OpenAPI,
  AsyncAPI, …) — those use their own schemas, per framework §4.4.

## Acceptance

- TC-910 — validate passes on a conformant contract (with a soft warning).
- TC-911 — validate flags a broken trace via the crown rule (exit 1).
- TC-912 — show + list render the contract.
- TC-913 — export emits Turtle with synthesised verification/work-unit links.
- TC-914 — init scaffolds a contract that validates; refuses to clobber.
- TC-915 — validate without a contract file is a clear error.
