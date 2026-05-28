---
id: FT-036
title: Lifecycle Gate
phase: 1
status: complete
depends-on: []
adrs:
- ADR-009
- ADR-013
- ADR-021
- ADR-032
- ADR-034
tests:
- TC-440
- TC-441
- TC-442
- TC-443
- TC-444
- TC-445
- TC-446
- TC-447
domains:
- data-model
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

## Description

Enforce the lifecycle ordering invariant: a feature cannot reach `complete` while any linked ADR is still `proposed`. This prevents decisions from being rubber-stamped after implementation, which defeats the purpose of ADRs as governing documents.

### Validation Rules

**W017 — `product graph check`:** Warning when a feature with `status: in-progress` or `complete` has a linked ADR with `status: proposed`. Exit code 2 (warning). Fires across the entire graph on every check.

**E016 — `product verify`:** Hard gate. Before running any TCs, verify checks all linked ADRs. If any is `proposed`, emit E016 and exit 1 without running tests or updating status.

### Invariant

```
∀f:Feature, ∀a:ADR where a ∈ f.adrs:
  f.status = "complete" → a.status ≠ "proposed"
```

Only `proposed` blocks. `accepted`, `superseded`, and `abandoned` all satisfy the invariant.

### Bypass

`product verify FT-XXX --skip-adr-check` suppresses E016 for migration scenarios (retroactively linking ADRs to existing features). Does not suppress W017 in graph check.

### Interaction with ADR-032

ADR-032 writes the content-hash at the moment of acceptance. This feature ensures acceptance happens before verify marks complete. Together they create the ordering: propose → accept (hash sealed) → implement → verify (gate checks acceptance) → complete.

---

## Functional Specification

### Inputs

- `product verify FT-XXX [--skip-adr-check]` — the feature ID to verify; `--skip-adr-check` bypasses the E016 gate for migration scenarios.
- `product graph check` — scans all features in the graph to detect W017 violations.
- The `adrs` list in each feature's YAML front-matter — the set of linked ADRs whose statuses are checked.
- The `status` field in each linked ADR's YAML front-matter — must be read to determine whether the invariant is satisfied.

### Outputs

- **`product verify FT-XXX` (gate triggered)** — exits with code 1 and prints E016 naming all linked ADRs with `status: proposed`. No TCs are run and no feature status is updated.
- **`product verify FT-XXX` (gate satisfied)** — proceeds normally; TCs are run and feature status is updated on pass.
- **`product graph check`** — emits W017 (exit code 2) for each feature in `in-progress` or `complete` status that has at least one linked ADR with `status: proposed`. Does not emit W017 for features with `status: planned`.

### State

Stateless. The gate reads ADR status fields from YAML front-matter on every `product verify` invocation and on every `product graph check` run. No cache or persistent record of ADR-check results is maintained.

### Behaviour

1. **Pre-verification ADR check** — before running any TCs, `product verify` loads the feature's `adrs` list, reads each linked ADR's `status` field, and collects all ADRs with `status: proposed`. If any are found, E016 is emitted and the command exits with code 1. The TC runner is never invoked.
2. **`--skip-adr-check` bypass** — suppresses E016 for migration scenarios (retroactively linking ADRs to features that predate the gate). The bypass is intentionally verbose to discourage casual use. It does not suppress W017 in `graph check`.
3. **`product graph check` — W017** — iterates all features; for each feature with `status: in-progress` or `status: complete`, checks every linked ADR. W017 is emitted once per feature that has at least one `proposed` ADR. Exit code 2 (warning-only).
4. **`planned` features exempt** — W017 does not fire for `planned` features because linking an ADR to a planned feature is legitimate forward-planning, not a lifecycle violation.
5. **Accepted, superseded, and abandoned ADRs satisfy the invariant** — only `proposed` status blocks completion. Superseded ADRs indicate a decision was made and later replaced; abandoned ADRs indicate the decision was not taken. Neither blocks verify.
6. **Interaction with ADR-032** — `product adr status ADR-XXX accepted` writes the content-hash at acceptance. This gate ensures acceptance happens before `product verify` marks the feature complete, establishing a clean ordering: propose → accept (hash sealed) → implement → verify (gate checks acceptance) → complete.

### Invariants

- `∀f:Feature, ∀a:ADR where a ∈ f.adrs: f.status = "complete" → a.status ≠ "proposed"` — no feature may reach `complete` while any of its linked ADRs has `status: proposed`.
- `product verify` must check all linked ADRs, not only the first. E016 names every proposed ADR in the diagnostic output.
- The gate runs before any TC execution — a feature with proposed ADRs never reaches a state where TCs pass but the status remains unchanged.

### Error handling

- **E016** — `product verify` blocked by a linked ADR with `status: proposed`. Exit code 1. Message names all proposed ADRs, states their current status, and hints to accept them or remove the link.
- **W017** — feature `in-progress` or `complete` with at least one linked `proposed` ADR. Exit code 2 (warning). Message names the feature, lists the proposed ADRs, and hints to accept them.
- If a linked ADR ID in the feature's `adrs` list does not resolve to a known ADR in the graph, `product graph check` already emits a broken-link error (separate from this feature); the lifecycle gate treats the unresolvable ADR as a graph-level error, not a lifecycle violation.

### Boundaries

- Only `product verify` and `product graph check` enforce this gate. `product feature status FT-XXX complete` (manual status change) is a deliberate escape hatch and does not trigger E016.
- The gate applies only to ADRs listed in the feature's `adrs` front-matter. ADRs linked to linked features (transitive ADRs) are not checked.
- `--skip-adr-check` does not suppress W017 in `product graph check` — the warning continues to fire in the graph view regardless of how `verify` was invoked.
- The gate is `cross-cutting`: it applies to every feature in every domain and phase.

## Out of scope

- Enforcing ADR acceptance before a feature transitions to `in-progress` — prototyping during decision review is legitimate. Only the `complete` transition is gated.
- Automatic ADR acceptance when all TCs pass — acceptance is a human decision, not a test outcome.
- Checking that ADR rationale is complete or correct before acceptance — that is a qualitative review concern, not a lifecycle gate.
- Enforcing ordering between multiple linked ADRs — all linked ADRs must simply be non-`proposed`; their relative acceptance order is not constrained.
