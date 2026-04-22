---
id: FT-002
title: Repository Layout
phase: 1
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-004
tests:
- TC-005
- TC-006
- TC-007
- TC-008
- TC-011
- TC-012
- TC-154
domains:
- data-model
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

```
/docs
  product.toml              ← repository config (name, prefix, phases)
  /features
    FT-001-cluster-foundation.md
    FT-002-products-iam.md
    FT-003-rdf-event-store.md
  /adrs
    ADR-001-rust-language.md
    ADR-002-openraft-consensus.md
  /tests
    TC-001-binary-compiles.md
    TC-002-raft-leader-election.md
    TC-003-raft-leader-failover.md
  /graph
    index.ttl               ← generated, never hand-edited
  checklist.md              ← generated, never hand-edited
```

Subdirectory names and file prefixes are configurable in `product.toml`. The layout above is the default.

---