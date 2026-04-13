---
id: TC-145
title: implement_blocked_by_preflight
type: scenario
status: unimplemented
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
---

FT-009 has preflight gaps. Run `product implement FT-009`. Assert exit 1, preflight error message, no agent invoked.