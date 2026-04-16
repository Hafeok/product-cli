---
id: TC-484
title: context measure-all prints summary not bundles
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

Given a repo with 3 features, when `product context --measure-all` is run, stdout contains an aggregate summary table (with "measured:", "mean:", "median:" lines) but does NOT contain the full bundle content (no "# Context Bundle:" headers). Individual bundle content is suppressed to avoid flooding stdout.