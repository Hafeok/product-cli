# Product Probabilistic Forecasting Specification

> Standalone reference for Monte Carlo cycle time forecasting.
> Uses git tag timestamps — no estimates, no story points, no front-matter fields.
> Depends on: product-planning-due-date-spec.md (started tag)

---

## Overview

`product forecast` answers: given how this team has actually delivered features
in this codebase, what is the probability of completing a feature or phase by
a given date?

The answer is derived entirely from git tags:

- `product/FT-XXX/started` — when work began (from product-planning-due-date-spec.md)
- `product/FT-XXX/complete` (or `complete-vN`) — when verification passed

Cycle time = `complete_timestamp - started_timestamp`

No estimates. No story points. No planning poker. The model learns from what
this team has actually delivered — not from what teams generally deliver, not
from generic benchmarks. The longer the project runs, the more accurate the
forecasts become.

---

## Data Collection

### Building the cycle time sample

```bash
# Product runs this internally on every forecast invocation
git tag | grep "^product/.*/complete"
```

For each complete feature with a `started` tag:

```
cycle_time(FT-001) = complete_tag_ts - started_tag_ts
                   = 2026-04-11T09:14Z - 2026-04-08T13:00Z
                   = 2.84 days
```

Features without a `started` tag are excluded from the sample — they have
no reliable start point. Features without a `complete` tag are in-progress
or planned — also excluded.

`complete-vN` tags: if a feature has been re-verified, the most recent
completion tag is used. The cycle time measures time to first acceptance
(original `started` → first `complete`), not time to last re-verification.
Re-verification reflects specification changes, not implementation time.

### Sample output

```
product forecast --show-data

  Cycle time sample (14 complete features):

  FT-001  2.84d    FT-002  5.12d    FT-003  3.21d
  FT-004  8.44d    FT-005  2.10d    FT-006  4.88d
  FT-007  1.95d    FT-008  11.32d   FT-009  3.67d
  FT-010  2.44d    FT-011  6.78d    FT-012  4.01d
  FT-013  3.55d    FT-014  7.22d

  Distribution:
    min:   1.95d
    p25:   2.87d
    p50:   4.04d
    p75:   6.45d
    p90:   9.72d
    max:  11.32d
    mean:  4.82d
    stddev: 2.61d

  Note: sample size 14 — forecasts have moderate confidence.
  Confidence improves as more features complete.
```

---

## Monte Carlo Simulation

### Single feature forecast

For a single feature, the simulation is:

```
for run in 1..N:
    sample cycle_time from historical distribution
    simulated_complete = today + cycle_time
    record simulated_complete
```

After N runs, the output is the empirical CDF of simulated completion dates:

```
product forecast FT-015

  Historical cycle times (14 complete features):
    p50: 4.0d  ·  p75: 6.5d  ·  p90: 9.7d

  Simulation: 10,000 runs

  FT-015 — Rate Limiting  [in-progress, started 2026-04-14]
  Elapsed so far: 2.3 days

  Completion probability:
    By 2026-04-21  (in 4d):  34%
    By 2026-04-25  (in 8d):  63%
    By 2026-04-28  (in 11d): 78%
    By 2026-05-01  (in 14d): 89%  ← due date
    By 2026-05-07  (in 20d): 97%

  P50 forecast: 2026-04-25
  P85 forecast: 2026-05-02

  Sample size: 14  (moderate confidence)
```

The due date appears in the table automatically if one is set on the feature.
"Elapsed so far" is today minus the `started` tag timestamp — the simulation
samples the remaining work, not the total work.

### Remaining work adjustment

If a feature is already in progress, the simulation adjusts for elapsed time.
Two modes:

**Optimistic (default):** remaining = `sample - elapsed`. If `sample < elapsed`,
the feature is already overdue on that run — logged as completing "today."
This models "we've been working this long, the feature is nearly done."

**Conservative:** sample the full distribution regardless of elapsed time.
This models "elapsed time tells us nothing about remaining work."

```
product forecast FT-015 --remaining-mode conservative
```

Both are honest — they model different beliefs about work in progress.
The default (optimistic) matches how most teams reason about in-flight work.

### Dependency chain forecast

For a chain of features connected by `depends-on` edges, each feature in
the chain is sampled sequentially — FT-015 can't start until its predecessors
complete in the simulation:

```
product forecast FT-015 --with-dependencies

  Dependency chain: FT-001 (complete) → FT-003 (complete) → FT-015 (in-progress)

  Only FT-015 is uncertain — predecessors are already complete.

  [same output as single feature forecast]
```

```
product forecast FT-020 --with-dependencies

  Dependency chain:
    FT-015  in-progress  (started 2026-04-14, 2.3d elapsed)
    FT-018  planned      (depends on FT-015)
    FT-020  planned      (depends on FT-018)

  Simulation: 10,000 runs

  FT-020 completion probability:
    By 2026-05-07:  18%
    By 2026-05-15:  52%
    By 2026-05-22:  74%
    By 2026-06-01:  89%

  P50 forecast: 2026-05-15
  P85 forecast: 2026-05-25

  Critical chain: FT-015 → FT-018 → FT-020  (highest variance)
```

### Phase forecast

For a phase, the simulation models all features in the phase. The phase
completes when all its exit-criteria TCs are passing — which requires all
features contributing to those TCs to be complete:

```
product forecast --phase 2

  Phase 2 — Products and IAM  [LOCKED]
  Features: FT-009, FT-010, FT-011, FT-012, FT-013 (all planned)

  Simulation: 10,000 runs
  Method: features start as predecessors complete (dependency-aware)

  Phase 2 complete:
    By 2026-05-15:  12%
    By 2026-06-01:  48%
    By 2026-06-15:  73%
    By 2026-07-01:  91%

  P50 forecast: 2026-06-02
  P85 forecast: 2026-06-18

  Critical chain: FT-009 → FT-012 → FT-013  (highest cumulative variance)
  Bottleneck: FT-012 (highest individual variance in chain)

  Due dates in this phase:
    FT-009  due 2026-05-01  →  P(hit):  8%   ⚠
    FT-010  due 2026-05-15  →  P(hit): 31%
```

Due dates for features in the phase appear with their hit probability, flagging
the ones at risk.

---

## Sampling Method

The historical cycle time distribution is sampled using **bootstrap resampling**
(sampling with replacement from the actual observed values). This is preferred
over fitting a parametric distribution (log-normal, Weibull, etc.) because:

- No assumption about the shape of the distribution
- Naturally captures outliers and multi-modal behaviour
- Requires no parameter estimation
- Honest about small sample sizes

With 5–10 features in the sample, bootstrap resampling reflects the actual
observed variance including those anomalous 11-day features. A parametric fit
might smooth them out and underestimate tail risk.

Minimum sample size: 3 complete features. Below this, forecasting is disabled:

```
product forecast FT-015

  ✗ Insufficient data for forecasting.
  Only 2 complete features with cycle time data.
  Forecasting requires at least 3.

  Complete more features to enable forecasting.
```

---

## Confidence and Sample Size

Every forecast output includes a confidence note:

| Sample size | Note |
|---|---|
| 3–5 | "Very low confidence — based on only N features." |
| 6–10 | "Low confidence — forecasts have wide uncertainty." |
| 11–20 | "Moderate confidence." |
| 21–50 | "Good confidence." |
| 51+ | "High confidence." |

The confidence note is informational. The percentile outputs are always shown
— the developer decides how much to trust them.

---

## Commands

```bash
product forecast FT-015                       # single feature
product forecast FT-015 --with-dependencies   # include dependency chain
product forecast FT-020 --with-dependencies   # chain to root
product forecast --phase 2                    # full phase
product forecast --phase 2 --all-phases       # all phases in sequence
product forecast --show-data                  # show raw cycle time sample
product forecast --runs 50000                 # more runs for precision (default: 10000)
product forecast --remaining-mode conservative
product forecast FT-015 --by 2026-05-01       # P(complete by date)
```

### `--by DATE`

Quick probability query:

```
product forecast FT-015 --by 2026-05-01

  P(FT-015 complete by 2026-05-01): 47%
```

Useful for answering "what are the odds of hitting this commitment?"

---

## `product.toml` Configuration

```toml
[forecasting]
default-runs = 10000             # Monte Carlo iterations (increase for precision)
min-sample-size = 3              # minimum complete features for forecasting
remaining-mode = "optimistic"    # optimistic | conservative
```

---

## Session Tests

```
ST-320  cycle-time-computed-from-started-and-complete-tags
ST-321  features-without-started-tag-excluded-from-sample
ST-322  features-without-complete-tag-excluded-from-sample
ST-323  complete-v2-uses-first-complete-for-cycle-time
ST-324  show-data-displays-distribution-statistics
ST-325  forecast-single-feature-outputs-percentiles
ST-326  forecast-adjusts-for-elapsed-time-optimistic
ST-327  forecast-conservative-ignores-elapsed
ST-328  forecast-with-dependencies-chains-correctly
ST-329  forecast-phase-runs-all-features
ST-330  forecast-phase-shows-due-date-hit-probability
ST-331  forecast-by-date-outputs-single-probability
ST-332  insufficient-data-below-min-sample
ST-333  minimum-sample-size-configurable
ST-334  confidence-note-scales-with-sample-size
ST-335  critical-chain-identified-by-variance
ST-336  bottleneck-identified-correctly
```

---

## Invariants

- `product forecast` is a pure read operation. It never writes to disk,
  never creates tags, never modifies front-matter.
- The cycle time sample is derived exclusively from git tags. No front-matter
  fields, no configuration, no manual estimates.
- Bootstrap resampling is used. No parametric distribution is fitted.
  The raw observed cycle times are the model.
- Forecasts are always expressed as probabilities, never as point estimates
  presented as certain. The P50/P85 labels make the probabilistic nature
  explicit.
- The `started` tag is created at most once per feature (per
  product-planning-due-date-spec.md). The cycle time sample is therefore
  stable — adding more in-progress features does not change the historical
  distribution until they complete.
- Forecasting is disabled below `min-sample-size` complete features.
  Product never extrapolates from fewer than 3 data points.
