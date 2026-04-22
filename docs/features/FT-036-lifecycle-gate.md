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
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
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