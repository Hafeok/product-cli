---
id: TC-066
title: atomic_write_content
type: scenario
status: passing
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
runner: cargo-test
runner-args: "tc_066_atomic_write_content"
---

write a feature file atomically. Assert the file contains the expected content. Assert no `.product-tmp.*` files remain.