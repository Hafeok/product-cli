---
id: FT-126
title: product deliverable and release — the delivery layer over slices
phase: 6
status: complete
depends-on:
- FT-124
adrs:
- ADR-067
tests:
- TC-967
- TC-968
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — new `deliverable` + `release` subcommand families; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Validation lives in pure `pf::deliverable`/`pf::release`; the CLI adapters own file I/O.
  ADR-048: Read the slice/deliverable artifact sets; write only the new pointer file on `new`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd; `pf::deliverable`/`pf::release` carry unit tests over validation. No property or session dimension.
  ADR-040: Delivery units are §7.1 subgraphs of the What; they compose slices and touch no verification gate.
patterns:
- PAT-001
---

## Description

§7.1 names two delivery units: a **feature** (the smallest valuable slice) and a
**release** (a coherent set of features that ship together) — both subgraphs of
the What. FT-124 added the **slice** (the event-model section). This feature adds
the two delivery units on top:

- **deliverable** — the framework's delivery feature, pointing at exactly one
  slice plus its agreed acceptance criteria. (Named `deliverable` because
  `product feature` owns the legacy FT-XXX graph.)
- **release** — a set of deliverables that ship together.

The hierarchy: **release → deliverables → slice → event-model section.** Each
layer references the one below by id and restates nothing.

## Functional Specification

### Behaviour

- `product deliverable new <id> --slice <slice> [--accept "id:statement"…]` —
  validate the slice resolves to a saved slice, then write
  `.product/deliverables/<id>.yaml`. `list` / `show` surface them.
- `product release new <id> --feature <deliverable>…` — validate every member
  resolves to a saved deliverable, then write `.product/releases/<id>.yaml`.
  `list` / `show` surface them.
- A dangling reference (a slice or deliverable that does not exist) is rejected
  with exit 1; nothing is written.
- `product status` counts deliverables + releases in its Delivery line.

### Error handling

- `deliverable new` whose `--slice` does not resolve to a saved slice is rejected
  (exit 1); `release new` whose member does not resolve to a saved deliverable is
  likewise rejected. Nothing is written on rejection.
- `show`/`done` on an unknown id fails with a clear not-found error.

## Out of scope

- The §7.2 predicates — `feature_done` (every concept/flow realised + verified,
  acceptance passing) and `release_done` (all members done + the cut is closed) —
  are a separate increment: they need What-graph verification status and a
  directed-dependency closure, neither of which exists yet.

## Acceptance

- TC-967 — the full chain (slice → deliverable → release) is created and shows
  in `product status`.
- TC-968 — dangling slice/deliverable references are rejected.
