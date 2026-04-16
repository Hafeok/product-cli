---
id: TC-480
title: graph stats shows bundle token summary
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

Given a repo with 3 features, 2 of which have `bundle` blocks in front-matter (from prior `--measure` runs), when `product graph stats` is run, the output includes a "Bundle size" section showing measured count, mean, median, p95, max (with feature ID), and min (with feature ID) token values, plus threshold breach lines.