---
id: TC-017
title: context_bundle_no_frontmatter
type: scenario
status: passing
validates:
  features:
  - FT-011
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_017_context_bundle_no_frontmatter"
last-run: 2026-04-14T13:57:28.405167723+00:00
---

assert the context bundle output contains no YAML front-matter blocks (front-matter is stripped from all sections).