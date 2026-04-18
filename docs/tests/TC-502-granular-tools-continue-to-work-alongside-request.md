---
id: TC-502
title: granular tools continue to work alongside request interface
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_502_granular_tools_continue_to_work_alongside_request_interface
last-run: 2026-04-18T10:42:07.406621710+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 14.

**Setup:** fixture with at least one feature, one ADR, and one TC.

**Act:** interleave granular-tool calls and request applies:
1. `product feature new "coexist test"` → creates FT-N via the granular path
2. `product request apply` a request that creates an ADR and links it to FT-N via `changes`
3. `product feature domain FT-N --add api` → granular domain tool
4. `product request apply` a request that changes FT-N's status to `in-progress`
5. `product test runner TC-M --runner cargo-test --args "..."` → granular runner tool

**Assert:**
- All five operations succeed
- After all five, `graph check` exits 0
- FT-N's final state reflects: title from step 1, ADR linkage from step 2, domain from step 3, status from step 4
- TC-M's runner config from step 5 is intact
- Concurrent invocations of a granular tool and a request apply serialise through the advisory lock (ADR-015) — neither corrupts the other
- No deprecation warning is emitted by any granular tool