---
id: TC-409
title: AGENT.md contains current front-matter schemas
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Run `product agent-init`. Assert the generated `AGENT.md` contains a "Front-Matter Schemas" section with subsections for feature, ADR, test criterion, and dependency schemas. Assert schema content matches `product schema --all` output.
