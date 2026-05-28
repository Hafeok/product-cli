---
id: FT-040
title: Aggregate Bundle Metrics
phase: 1
status: complete
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
- TC-680
domains:
- api
- observability
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
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

---

## Description

Two additions that close the gap between per-feature bundle measurement (`product context FT-XXX --measure`) and repository-wide bundle size visibility: (1) a token summary section appended to `product graph stats`, and (2) a `--measure-all` flag on `product context` that measures every feature in one pass and prints the aggregate table.

## Functional Specification

### Inputs

- `product graph stats` — no new flags; the token summary is appended automatically when any features have been measured.
- `product context --measure-all [--depth N]` — `--measure-all` flag; optional `--depth` (default 1). The `id` positional argument is not required when `--measure-all` is passed.
- `bundle` block in feature front-matter — written by prior `product context FT-XXX --measure` runs; read by `graph stats` and by `--measure-all` when combining with previously measured data.
- `metrics.jsonl` — appended with `bundle_measure` entries during `--measure-all`.
- `[metrics.thresholds]` in `product.toml` — `bundle_tokens_max` (default 12,000), `bundle_depth1_adr_max` (default 8), `bundle_domains_max` (default 6) — used for threshold breach counts in the summary.

### Outputs

- **`product graph stats` — token summary section:**
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
  When no features are measured, prints: `No bundle measurements — run product context --measure-all`.
- **`product context --measure-all`** — for each feature in ID order: assembles bundle, computes metrics, writes `bundle` block to feature front-matter, appends `bundle_measure` entry to `metrics.jsonl`. After all features are processed, prints the aggregate summary table (same format as the `graph stats` token section). Bundle content is NOT printed to stdout (unlike single-feature `--measure`).
- **W012** — emitted to stderr when unmeasured features exist (surfaced in the summary line).

### State

Per-feature bundle measurements are persisted in the `bundle` block of each feature's YAML front-matter by prior `product context --measure` or `--measure-all` runs. `product graph stats` reads these stored values without re-measuring. `metrics.jsonl` accumulates `bundle_measure` entries over time; the p95 and other percentile statistics are derived from the stored feature front-matter values at `graph stats` invocation time, not from `metrics.jsonl` directly.

### Behaviour

1. **`product graph stats` token section** — after rendering the existing centrality and graph health output, `graph stats` iterates all features, collects those with a `bundle` block in front-matter, and computes mean, median, p95, max, and min over `tokens-approx` values. Threshold breach counts use the configured or default thresholds from `[metrics.thresholds]`. Features without a `bundle` block are counted as unmeasured; W012 is emitted if any exist.
2. **`--measure-all` iteration** — features are iterated in ID order. For each, the bundle is assembled at the specified depth (default 1), metrics are computed (token count, ADR count, domain count), the `bundle` block is written to feature front-matter via atomic write, and a `bundle_measure` entry is appended to `metrics.jsonl`. Errors on individual features are logged to stderr and do not abort the pass.
3. **No stdout bundle content** — `--measure-all` does not print bundle content to stdout. Only per-feature progress lines and the final summary table are printed.
4. **Aggregate summary** — after all features are measured, `--measure-all` calls the same aggregate computation as `graph stats` and prints the same table format. This is the primary intended way to establish a baseline.
5. **Threshold configuration** — `bundle_tokens_max`, `bundle_depth1_adr_max`, and `bundle_domains_max` are read from `[metrics.thresholds]` in `product.toml`. When absent, built-in defaults apply (12,000 tokens, 8 ADRs, 6 domains).

### Invariants

- `product context --measure-all` writes `bundle` blocks to front-matter atomically — partial failure on one feature does not corrupt others.
- The aggregate summary produced by `--measure-all` and the token section in `graph stats` use the same computation; they must produce consistent statistics for the same set of measured features.
- W012 fires whenever at least one feature lacks a `bundle` block; it is a warning (exit 2), not a hard error.
- The `id` argument is not required when `--measure-all` is passed; passing both is an error.

### Error handling

- **`--measure-all` with an explicit feature ID** — `ProductError::ConfigError` naming the conflict.
- **Individual feature assembly failure during `--measure-all`** — printed to stderr; the pass continues with the remaining features. Exit code reflects the worst outcome (0 if all succeeded, 2 if any warnings, 1 if any hard errors).
- **No features in the graph** — `graph stats` token section prints "No features measured" and exits 0.
- **Missing `metrics.jsonl`** — `--measure-all` creates the file if absent; `graph stats` does not require it (reads from front-matter only).
- **Threshold keys absent from `product.toml`** — built-in defaults are used silently; no error or warning.

### Boundaries

- `product graph stats` token section is read-only — it does not write or update any front-matter.
- `--measure-all` writes front-matter and `metrics.jsonl`; it does not print bundle content to stdout and is not a substitute for `product context FT-XXX` when the bundle content itself is needed.
- The aggregate statistics (mean, median, p95, max, min) are computed over `tokens-approx` values only. Other bundle block fields (ADR count, domain count) are used for threshold breach reporting but not for the distribution statistics.
- Per-feature measurement history is not maintained — each `--measure` or `--measure-all` run overwrites the `bundle` block in front-matter with the latest measurement.

## Out of scope

- Trend analysis over time — that is `product metrics trend` (ADR-024). `--measure-all` establishes snapshots; trend comparison requires `metrics.jsonl` time series analysis.
- Per-model bundle size breakdown — `product context --measure` uses a single token approximation method; per-model templates are covered by ADR-049.
- Automatic `--measure-all` on every `graph stats` invocation — measurement is an explicit opt-in operation.
- Pruning or summarising `metrics.jsonl` — the file grows indefinitely; rotation or compaction is not in scope.
