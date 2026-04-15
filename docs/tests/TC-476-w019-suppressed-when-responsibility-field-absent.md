---
id: TC-476
title: W019 suppressed when responsibility field absent
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Given** a repository without `[product].responsibility` in product.toml AND features exist with any titles
**When** `product graph check` runs validation
**Then** no W019 warnings are emitted for any feature — the check is inert when responsibility is absent