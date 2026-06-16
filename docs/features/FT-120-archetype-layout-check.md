---
id: FT-120
title: product archetype check — enforce a layout model against the repository tree
phase: 6
status: complete
depends-on:
- FT-114
adrs:
- ADR-060
tests:
- TC-945
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `archetype check` subcommand; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The filesystem walk lives in a pure `pf::layout_check` slice; the CLI is a thin BoxResult adapter.
  ADR-048: Reads an archetype's `layout.yaml` + the repository tree; writes nothing.
  ADR-051: The TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: One scenario TC drives the binary through assert_cmd; `pf::layout_check` carries unit tests over tempdir trees. No property or session dimension for a filesystem walk.
  ADR-040: Layout conformance is the cheapest §6.2 gate; it composes the existing layout model, not the verify pipeline.
patterns:
- PAT-001
---

## Description

FT-114 introduced the §4.3 **layout model** and validated that the model is
*well-formed*. It explicitly did not enforce the model against an actual
repository tree. This feature closes that gap: `product archetype check <name>`
applies an archetype's `layout.yaml` to the files on disk — the cheapest §6.2
gate, run first, before any expensive realisation work.

## Functional Specification

### Inputs

- An archetype name, resolving to `.product/archetypes/<name>/layout.yaml`.
- The repository tree rooted at the discovered product root.

### Behaviour

`product archetype check <name>` loads the archetype's layout model and applies
each glob rule to the tree (`pf::layout_check`):

- `must_exist` — the glob must match, honouring `cardinality` (`exactly 1`,
  `at least 1`) and the `for_each` "1 per scope" form (one match per directory
  matched by the scope).
- `must_not_exist` — a prohibition: the glob must match nothing.
- `must_co_exist` — within each directory matching `when`, every `require`
  sibling must be present.
- `no_orphans` — every file under the scope must be matched by some
  `may_exist_here`/`must_exist` allow rule (allowlist semantics): the
  *unanticipated* file is the failure.

A trailing `/**` is normalised to match files. Exits 0 with `layout-conformant`
when every rule holds; exits 1 listing each violation (`[focus] path: message`)
otherwise.

### Error handling

- A missing archetype directory is a clear error pointing at
  `product archetype init`.
- An archetype with no `layout.yaml` reports that there is nothing to check and
  exits 0.

## Out of scope

- It does not interpret file *contents*; globs match paths, not meaning.
- It does not fix violations or move files; it reports.

## Acceptance

- TC-945 — apply a layout model to the tree: conformant passes, a forbidden
  file fails with the `must_not_exist` violation.
