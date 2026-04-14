---
id: TC-040
title: context_bundle_formal_blocks_preserved
type: scenario
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
runner: cargo-test
runner-args: "tc_040_context_bundle_formal_blocks_preserved"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

assert that formal blocks in test criteria are preserved verbatim in the context bundle output, not stripped like front-matter.