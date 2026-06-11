---
id: ADR-052
title: Two Pillars clauses are checked structurally from the knowledge graph
status: accepted
features:
- FT-108
supersedes: []
superseded-by: []
domains:
- api
- error-handling
scope: feature-specific
content-hash: sha256:6914a1e2d7d1e0d490a47dc9edcb06af23362d3f24112f46af998ef6a9555825
---

## Context

The Two Pillars specification (working draft 0.1) names Product as a
framework implementation of its specification pillar: features carry the
What, ADRs carry the How, TCs carry the declared acceptance criteria, and
`product verify` is the declared judge producing verdicts. The spec
defines conformance as satisfying the normative clauses of a target
autonomy profile (Level 3: `SPEC-WHAT-*`, `SPEC-HOW-*`, `SPEC-SPMC-*`,
`SPEC-DERIVE-*`). Until now, nothing in the toolchain could say whether a
Product repository actually satisfies those clauses — conformance was
asserted, not measured.

## Decision

Conformance is evaluated **structurally, from the knowledge graph, against
an explicit clause registry** in a pure `conformance` slice
(`product-core/src/conformance/`), surfaced as `product conformance check`.

- Each spec clause that is mechanically decidable from front-matter,
  bodies, and config maps to one check function keyed by the spec's own
  stable clause identifier (`SPEC-WHAT-5`, `SPEC-DERIVE-3`, …).
- Clauses guaranteed by the artifact model or graph loader (`SPEC-SPLIT-1`
  separation, `SPEC-HOW-2.2` acyclicity via E003/E004) are reported as
  passing **by construction**, so the report shows full clause coverage
  rather than a silent subset.
- The spec's keyword strength is preserved: a violated MUST is a
  `violation` (exit 1); a disregarded SHOULD is an `advisory` (exit 0).
  The profile verdict is `level-3` exactly when no MUST is violated.
- Clauses requiring semantic judgment or Level 4/5 execution machinery are
  *not* checked; the full clause-by-clause disposition is documented in
  `docs/two-pillars-conformance.md` so unchecked clauses are declared, not
  hidden — the same explicit-over-implicit principle the spec's
  SPEC-DERIVE-3 demands of How elements.
- EXEC-CLOSE-4 ("every output judged before acceptance") is included even
  though it is an execution-pillar clause, because Product's own
  completion semantics already imply it: a `complete` feature with a
  non-passing, non-acknowledged TC verdict is a closure violation the
  graph can detect today.

## Rationale

Structural checking from the graph keeps the gate deterministic, fast, and
free of LLM calls — the same trade ADR-019 made for gap analysis. Keying
findings by the spec's clause IDs makes every finding citable against the
external document and keeps the registry auditable: a reader can diff the
registry against the spec's clause list and see precisely what is covered.
Reusing the spec's MUST/SHOULD strength as the severity model means the
exit-code contract follows from the spec itself rather than from a local
severity invention.

## Rejected alternatives

- **Extend `product gap check` with G0xx rules for spec clauses.** Gap
  analysis answers "is this ADR internally complete?"; conformance answers
  "does this repository satisfy an external specification's profile?". The
  outputs differ (per-clause table + profile verdict vs per-ADR findings),
  the severity models differ (MUST/SHOULD vs high/medium/low), and fusing
  them would bury the profile verdict inside ADR-scoped reports.
- **LLM-driven semantic conformance review.** Rejected for the gate: it is
  non-deterministic, slow, and unverifiable in CI. Semantic clauses are
  declared out of scope in the mapping document instead; an LLM review can
  layer on top later the way D001/D002 layer over structural drift.
- **A standalone linter script outside the CLI.** Rejected: the checks
  need the parsed graph, config discovery, and body-section parser the
  core library already owns; a script would re-implement the parser and
  drift from it.
- **Suppression baseline from day one.** Rejected: gap and drift baselines
  exist to manage pre-existing debt during adoption; conformance findings
  on this repository are actionable directly. A baseline can be added
  later without breaking the report shape.

## Test coverage

- TC-891 — clean pass with Level 3 verdict on a conforming repository.
- TC-892 — SPEC-WHAT-5 / SPEC-WHAT-8 violations on an incomplete What unit.
- TC-893 — SPEC-DERIVE-3 violation for an unanchored feature-specific ADR.
- TC-894 — EXEC-CLOSE-4 violation for a complete feature with a failing
  TC; `unrunnable` acknowledged-skip accepted.
- TC-895 — JSON report shape: `spec`, `profile`, `clauses[]`, `findings[]`.
