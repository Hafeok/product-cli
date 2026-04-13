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
---

assert that formal blocks in test criteria are preserved verbatim in the context bundle output, not stripped like front-matter.