---
id: FT-031
title: Code Structure and Quality Standards
phase: 3
status: complete
depends-on: []
adrs:
- ADR-029
- ADR-001
- ADR-043
tests:
- TC-369
- TC-370
- TC-371
- TC-372
- TC-373
- TC-374
- TC-375
- TC-376
- TC-377
- TC-378
- TC-379
- TC-380
- TC-402
domains: []
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

## Description

Enforce structural quality rules with measurable thresholds (ADR-029): file size limits (400 lines hard, 300 warning), function length limits (40 statement lines), mandatory module decomposition, and single-responsibility doc comments on every source file. Checked by shell scripts in `scripts/checks/` and run via `product verify --platform`.
