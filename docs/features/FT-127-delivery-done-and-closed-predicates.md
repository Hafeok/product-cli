---
id: FT-127
title: product deliverable/release done — the §7.2 computed delivery predicates
phase: 6
status: complete
depends-on:
- FT-126
adrs:
- ADR-068
tests:
- TC-969
- TC-975
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — `done`/`accept` subcommands on existing families; nothing removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The predicates live in the pure `pf::done` slice; the CLI adapters own I/O.
  ADR-048: Read the What graph + slice/deliverable/decider artifacts; `accept` writes the deliverable's recorded verdict.
  ADR-051: Every TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd; `pf::done` carries unit tests over feature_done/release_done/cut_closed. No property or session dimension.
  ADR-040: "`done` composes the existing verifications (domain conformance, Decider simulation) + acceptance; it adds no new verification kind."
patterns:
- PAT-001
---

## Description

§7.2 makes "done" a computed predicate, not a judgement: a feature is done when
its in-scope What elements pass their verifications and its acceptance criteria
pass; a release is done when all its features are done and its cut is closed.
This feature computes those predicates from verifications that already exist —
no estimation.

## Functional Specification

### feature_done (`product deliverable done <id>`)

Computes a verdict + per-check breakdown + progress fraction over the
deliverable's slice scope (the What subgraph its slice covers):

- **domain** — every in-scope element passes `validate_graph` (domain
  conformance, §6.3).
- **behavioural** — every Decider over an in-scope aggregate is sound + complete
  (the §3.3 simulation gate).
- **acceptance** — every acceptance criterion is recorded `passing`.

`done` is true iff every check passes. Exits 0 when done, 1 otherwise (a gate).
A criterion's verdict is recorded with `product deliverable accept <id>
<criterion> --pass|--fail`.

### release_done (`product release done <id>`)

`done` iff **all member features are done AND the cut is closed**. The cut is
closed when no in-scope node depends (via a directed graph edge) on a node
outside the release's union of slice scopes; open edges are listed. Exits 0/1.

### Honesty

Done is exactly as honest as the verifications are strong. It composes the
checks the toolchain can actually run today (domain conformance, Decider
simulation, recorded acceptance). Realisation tracking and post-realisation
behavioural conformance (`decider conform`) tighten it further as they are
recorded.

## Out of scope

- Persisting `decider conform` (post-realisation) verdicts as a done input — the
  behavioural gate here is the pre-realisation simulation.

## Acceptance

- TC-969 — a deliverable with pending acceptance is not done; recording the
  verdict makes it done.
- TC-975 — a release is done only when its members are done and its cut is closed.
