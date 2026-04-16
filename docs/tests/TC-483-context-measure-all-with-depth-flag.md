---
id: TC-483
title: context measure-all with depth flag
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

Given a repo with features, when `product context --measure-all --depth 2` is run, the bundle assembly uses depth 2 for BFS traversal, and the resulting `bundle` blocks reflect the wider context.