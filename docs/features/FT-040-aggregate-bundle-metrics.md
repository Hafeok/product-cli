---
id: FT-040
title: Aggregate Bundle Metrics
phase: 1
status: planned
depends-on: []
adrs:
- ADR-006
- ADR-012
- ADR-024
tests:
- TC-480
- TC-481
- TC-482
- TC-483
- TC-484
- TC-485
domains:
- api
- observability
domains-acknowledged: {}
---

## Summary

Two additions to close the gap between per-feature bundle measurement (`product context FT-XXX --measure`) and repository-wide bundle size visibility:

1. **Token summary in `product graph stats`** — show aggregate bundle size statistics (mean, median, p95, max, min) and threshold breaches for all measured features.
2. **`product context --measure-all`** — measure every feature in one pass, writing all bundle blocks and metrics.jsonl entries, then printing the aggregate table.

## Motivation

Per-feature measurement exists (ADR-006 `--measure` flag) but there's no single command to measure all features at once, and `product graph stats` shows centrality stats but not the aggregate token view that ADR-024 defines. Running `product context FT-XXX --measure` 39 times is impractical for baseline establishment or periodic audits.

## Specification

### 1. Token summary in `product graph stats`

After the existing centrality summary, `product graph stats` appends:

```
Bundle size (tokens-approx):
  measured:    12 / 15 features  (3 unmeasured — W012)
  mean:        5,840 tokens
  median:      5,200 tokens
  p95:         10,800 tokens
  max:         11,200 tokens  FT-003
  min:         2,100 tokens   FT-011

  Over token threshold (>12,000):   0 features
  Over ADR threshold (>8 ADRs):     1 feature  — FT-003
  Unmeasured:                       3 features  — FT-013, FT-014, FT-015
```

- Reads `bundle` block from feature front-matter (written by `--measure`)
- Token threshold from `[metrics.thresholds.bundle_tokens_max]` in product.toml
- ADR threshold from `[metrics.thresholds.bundle_depth1_adr_max]` in product.toml
- If no features are measured, prints "No bundle measurements — run `product context --measure-all`"
- Warning W012 emitted to stderr when unmeasured features exist

### 2. `product context --measure-all`

```bash
product context --measure-all          # measure all features at depth 1
product context --measure-all --depth 2  # measure all features at depth 2
```

- Iterates all features in ID order
- For each: assembles bundle, computes metrics, writes `bundle` block to feature front-matter, appends to metrics.jsonl
- Bundle content is NOT printed to stdout (unlike single-feature `--measure`)
- After all features are measured, prints the aggregate summary table (same format as graph stats token section)
- Exit code 0 on success
- The `id` argument is not required when `--measure-all` is passed

## Thresholds

Uses existing threshold config from ADR-024:
- `bundle_tokens_max` — per-feature token ceiling (default warning at 12,000)
- `bundle_depth1_adr_max` — per-feature ADR count ceiling (default warning at 8)
- `bundle_domains_max` — per-feature domain count ceiling (default warning at 6)