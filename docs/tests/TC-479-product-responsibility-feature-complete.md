---
id: TC-479
title: product responsibility feature complete
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

FT-039 is complete when all of the following hold:

1. **Config parsing** — `ProductConfig` deserializes `[product].responsibility` from product.toml and falls back gracefully when absent (TC-472)
2. **MCP tool** — `product_responsibility` MCP tool returns the product name and responsibility statement, or errors when not configured (TC-473)
3. **Context bundle** — `product context FT-XXX` includes `product` and `responsibility` in the bundle header when configured, omits them when not (TC-474, TC-477)
4. **Graph validation** — `product graph check` emits W019 for features outside the declared responsibility, and suppresses W019 when responsibility is absent (TC-475, TC-476)
5. **Single-statement invariant** — validation warns on top-level conjunction in the responsibility statement (TC-478)
6. **ADR amendments recorded** — ADR-006, ADR-013, and ADR-022 have been amended with audit trail entries per ADR-032
7. **All TCs passing** — TC-472 through TC-478 are in `passing` status