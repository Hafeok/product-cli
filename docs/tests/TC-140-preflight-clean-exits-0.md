---
id: TC-140
title: preflight_clean_exits_0
type: exit-criteria
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_140_preflight_clean_exits_0"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

feature with all cross-cutting ADRs linked and all declared domains covered. Assert `product preflight FT-XXX` exits 0 and prints "Pre-flight clean."