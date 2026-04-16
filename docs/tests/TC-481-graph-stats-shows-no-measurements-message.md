---
id: TC-481
title: graph stats shows no measurements message
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

Given a repo with features that have NO `bundle` blocks, when `product graph stats` is run, the output includes the line "No bundle measurements" suggesting the user run `product context --measure-all`.