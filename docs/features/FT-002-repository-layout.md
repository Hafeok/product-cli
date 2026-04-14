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
- TC-186
- TC-187
- TC-188
- TC-189
- TC-192
- TC-193
domains: []
domains-acknowledged: {}
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