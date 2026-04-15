---
id: TC-442
title: graph check emits W017 for complete feature with proposed ADR
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_442_graph_check_emits_w017_for_complete_feature_with_proposed_adr"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature with `status: complete` linked to an ADR with `status: proposed`. Run `product graph check`. Assert:

1. Output contains `warning[W017]` naming the feature and the proposed ADR.
2. Exit code is 2 (warnings only, per ADR-009).
3. The warning message includes a hint to accept the ADR or remove the link.

Also verify W017 fires for `status: in-progress` features with proposed ADR links.