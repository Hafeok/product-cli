---
id: TC-145
title: implement_blocked_by_preflight
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_145_implement_blocked_by_preflight"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

FT-009 has preflight gaps. Run `product implement FT-009`. Assert exit 1, preflight error message, no agent invoked.