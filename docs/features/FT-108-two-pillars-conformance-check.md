---
id: FT-108
title: Two Pillars conformance check over the knowledge graph
phase: 6
status: complete
depends-on: []
adrs:
- ADR-052
tests:
- TC-891
- TC-892
- TC-893
- TC-894
- TC-895
domains:
- api
- error-handling
domains-acknowledged:
  ADR-041: Additive feature — no CLI surface, MCP tool, schema field, or behaviour is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: No context bundle or template change; the command does not render bundles.
  ADR-043: Followed — pure `conformance` slice in product-core (check/render/tests) with a thin BoxResult adapter in commands/conformance.rs (exit-code semantics).
  ADR-048: Read-only command — no new state files; nothing written under .product/ or the repo root.
  ADR-051: All five TCs declare `observes:` (exit-code, stdout) and their bodies assert on those named surfaces.
  ADR-018: Five scenario TCs drive the binary through the assert_cmd harness; the slice's pure functions carry 18 unit tests. No property or session dimension for a read-only reporter.
  ADR-040: The check is purely structural — no LLM boundary is crossed and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

The Two Pillars specification (working draft 0.1) defines what a What
specification, a How specification, and their derivation links MUST contain
for spec-driven (Level 3) development. Product already *is* an
implementation of the specification pillar — features are What units, ADRs
are How units, TCs are declared acceptance criteria, and `product verify`
is the declared judge. What was missing is a way to *demonstrate*
conformance: a command that evaluates the knowledge graph against the
spec's clauses and reports, clause by clause, whether the repository
satisfies the Level 3 profile.

`product conformance check` closes that gap. It evaluates the mechanically
checkable subset of the Level 3 clause set (SPEC-SPLIT, SPEC-WHAT,
SPEC-HOW, SPEC-DERIVE, plus the EXEC-CLOSE-4 closure rule that `verify`
semantics already imply) and emits a per-clause report with violations
(broken MUSTs), advisories (disregarded SHOULDs), and a profile verdict.
The full clause-to-mechanism mapping, including clauses satisfied by
construction or out of scope at Level 3, lives in
`docs/two-pillars-conformance.md`.

## Functional Specification

### Inputs

- The knowledge graph (features, ADRs, TCs) rebuilt from front-matter.
- Project declarations from the product config: `name` (system identity,
  SPEC-WHAT-1), `[product].responsibility` (purpose, SPEC-WHAT-2), and the
  configured `[paths]` for features and ADRs (SPEC-SPLIT-1).
- `--format text|json` (also honours the global `--format json`).

### Outputs

- Text: a clause table (`[pass]` / `[FAIL]` per clause), a findings list
  with per-finding suggested actions, a summary line, and a Level 3
  verdict.
- JSON: a single report object with `spec`, `profile`
  (`level-3` | `below-level-3`), `scope`, `clauses[]`, `findings[]`, and
  `summary` — stable field names for CI consumption.

### State

Stateless and read-only. The command never writes artifacts, baselines, or
front-matter; the graph is rebuilt from disk on every invocation (ADR-003).

### Behaviour

Each clause in the registry is evaluated against the graph:

- SPEC-SPLIT-1 — features and ADRs are distinct artifact kinds; violated
  only when both `[paths]` entries point at the same directory.
- SPEC-WHAT-1 / SPEC-WHAT-2 — `name` and `[product].responsibility` are
  declared and non-empty.
- SPEC-WHAT-4 — every non-abandoned feature's Functional Specification has
  non-empty Behaviour plus Error handling subsections (behaviours declare
  their exception conditions).
- SPEC-WHAT-5 — every non-abandoned feature has a non-empty
  `## Out of scope` section.
- SPEC-WHAT-8 — every non-abandoned feature has at least one linked TC
  (declared in `tests:` or via `validates.features`).
- SPEC-HOW-2.1 (advisory) — an accepted ADR title with a top-level " and "
  suggests two fused decisions.
- SPEC-HOW-2.2 — dependency/supersession cycles; passes by construction
  because the graph loader rejects cycles (E003/E004) before checks run.
- SPEC-HOW-5 — every accepted ADR documents rejected alternatives.
- SPEC-DERIVE-3 — an accepted feature-specific ADR anchored to no feature
  is an undeclared product decision.
- EXEC-CLOSE-4 — a complete feature is accepted output; every linked TC
  must hold a `passing` verdict (or `unrunnable`, the acknowledged
  platform-skip verdict `product verify` itself accepts).

Findings carry the clause ID, severity, affected artifact, description,
and a suggested action. The profile verdict is `level-3` when no MUST
clause is violated; advisories never break conformance.

### Invariants

- The clause registry order is the report order; every registered clause
  appears in every report, passed or failed.
- A violated MUST is always a `violation`; a disregarded SHOULD is always
  an `advisory`; advisories never affect the exit code or profile.
- Abandoned features and non-accepted ADRs are exempt from per-artifact
  clauses — conformance is judged on the live specification surface.

### Error handling

- Exit 0: no violations (advisories permitted).
- Exit 1: at least one violation, after the full report is printed.
- Graph or config load failures surface the existing `ProductError`
  diagnostics and exit codes (E001, E024, …) unchanged.

### Boundaries

- The slice (`product-core/src/conformance/`) is pure: graph + project
  declarations in, report out. The CLI adapter owns I/O, format selection,
  and the exit code.
- The command checks specification artifacts, not generated code:
  Level 4/5 execution clauses (workers, capabilities, credentials) are out
  of report scope and documented as such in the mapping document.

## Out of scope

- Suppression baselines for conformance findings (gap/drift-style
  `gaps.json`); findings are recomputed on every run.
- An MCP tool mirror of the command (`product_conformance_check`).
- Checking clauses that need semantic judgment (e.g. SPEC-WHAT-2's "without
  inference" readability) — only structural presence is checked.
- Level 4/5 execution-contract enforcement (worker roles, capability
  grants, scoped credentials, outcome contracts).
