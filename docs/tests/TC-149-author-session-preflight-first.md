---
id: TC-149
title: author_session_preflight_first
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
runner-args: "tc_149_author_session_preflight_first"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

start `product author feature` for FT-009 with preflight gaps. Assert the first MCP tool call from the session is `product_preflight`, not a content scaffold call.