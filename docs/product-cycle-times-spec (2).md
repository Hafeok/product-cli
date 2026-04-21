# Product Cycle Time Visibility Specification

> Standalone reference for `product cycle-times` and `product forecast --naive`.
> Simple visibility into historical cycle times from git tags.
> No simulation, no probabilistic model — just the data.
>
> For teams wanting full Monte Carlo forecasting, the `started` and `complete`
> tags (per product-planning-due-date-spec.md) provide the raw data via git.
> Export the timestamps and run whatever model you prefer in your own tools.

---

## Philosophy

Product ships cycle time visibility, not cycle time prediction.

A probabilistic model that fits 14 data points produces forecasts with
confidence intervals wider than the forecast values themselves. The precision
implied by "P50 forecast: 2026-04-25" is false — the data doesn't support it
until the sample is much larger (50+ features), and by then teams have their
own analytics pipelines anyway.

What Product can honestly provide at any scale:

- **Historical cycle times.** Each complete feature's started-to-complete
  elapsed time, derived from git tags.
- **Descriptive statistics.** Median, range, trend direction.
- **A rough naive projection.** Clearly labelled as rough, bounded by min/max
  of recent samples rather than percentiles of a fitted distribution.

Teams that want more — Monte Carlo, bucket-by-complexity, correlation analysis —
export the tag timestamps and run their model in a tool designed for it.
`product cycle-times --format json` exists precisely for this.

---

## `product cycle-times`

Lists historical cycle times derived from `product/FT-XXX/started` and
`product/FT-XXX/complete` git tags.

### Default output

```
product cycle-times

  Feature   Started       Completed     Cycle time
  ─────────────────────────────────────────────────
  FT-001    2026-04-08    2026-04-11    2.84d
  FT-002    2026-04-12    2026-04-17    5.12d
  FT-003    2026-04-15    2026-04-18    3.21d
  FT-004    2026-04-17    2026-04-26    8.44d
  FT-005    2026-04-20    2026-04-22    2.10d
  FT-006    2026-04-22    2026-04-27    4.88d
  FT-007    2026-04-25    2026-04-27    1.95d
  FT-008    2026-04-28    2026-05-09   11.32d
  FT-009    2026-05-05    2026-05-08    3.67d
  FT-010    2026-05-08    2026-05-10    2.44d
  FT-011    2026-05-10    2026-05-17    6.78d
  FT-012    2026-05-15    2026-05-19    4.01d
  FT-013    2026-05-18    2026-05-21    3.55d
  FT-014    2026-05-20    2026-05-27    7.22d

  14 complete features.

  Recent 5:   median 4.01d   range 2.44 – 7.22d
  All:        median 4.02d   range 1.95 – 11.32d
  Trend:      stable         (recent ≈ historical)
```

### Fields

| Field | Source |
|---|---|
| Started | `product/FT-XXX/started` tag timestamp |
| Completed | Most recent `product/FT-XXX/complete` or `complete-vN` tag timestamp |
| Cycle time | Completed minus Started, in days (one decimal) |

Features without both tags are excluded from the output. In-progress features
are shown separately with a `--in-progress` flag.

### Trend indicator

Compares the median of the most recent 5 complete features to the median of
all features. One of:

- **accelerating** — recent median more than 25% below all-time median
- **stable** — recent median within 25% of all-time median
- **slowing** — recent median more than 25% above all-time median

No forecast, just direction. If a team has fewer than 6 complete features, the
trend indicator is omitted (insufficient history to distinguish recent from
all-time).

### Flags

```bash
product cycle-times                          # default output
product cycle-times --recent N               # only last N completed features
product cycle-times --phase 1                # scope to features in phase 1
product cycle-times --in-progress            # include in-progress features with elapsed-so-far
product cycle-times --format json            # machine-readable
product cycle-times --format csv             # for spreadsheet import
```

### JSON output

```json
{
  "features": [
    {
      "id": "FT-001",
      "started": "2026-04-08T13:00:00Z",
      "completed": "2026-04-11T09:14:22Z",
      "cycle_time_days": 2.84,
      "phase": 1
    }
  ],
  "summary": {
    "count": 14,
    "recent_5": { "median": 4.01, "min": 2.44, "max": 7.22 },
    "all":      { "median": 4.02, "min": 1.95, "max": 11.32 },
    "trend": "stable"
  }
}
```

### CSV output

```csv
feature_id,started,completed,cycle_time_days,phase
FT-001,2026-04-08T13:00:00Z,2026-04-11T09:14:22Z,2.84,1
FT-002,2026-04-12T10:30:00Z,2026-04-17T15:42:00Z,5.12,1
...
```

This is the export format for teams running their own forecasting models.
Timestamps are ISO 8601. Cycle time is numeric days with one decimal.

### In-progress features

```bash
product cycle-times --in-progress
```

```
product cycle-times --in-progress

  Feature   Started       Status        Elapsed
  ───────────────────────────────────────────────
  FT-015    2026-05-20    in-progress   2.3d
  FT-016    2026-05-21    in-progress   1.1d

  Reference: median cycle time (recent 5) is 4.01d
```

In-progress features have a `started` tag but no `complete` tag. Elapsed is
`now - started_tag_timestamp`. The reference line shows the recent median
for comparison — users can compare their current work to historical norms
without the tool making a prediction.

---

## `product forecast --naive`

A rough projection based on recent cycle times. Labelled as naive to signal
that it is not a probabilistic forecast.

### Output

```
product forecast FT-015 --naive

  FT-015 — Rate Limiting  [in-progress, started 2026-05-20]
  Elapsed: 2.3d

  Recent 5 complete features:
    median 4.01d  ·  range 2.44 – 7.22d

  Naive projection:
    Likely completion:  2026-05-24   (today + recent median − elapsed)
    Optimistic:         2026-05-22   (today + recent min − elapsed)
    Pessimistic:        2026-05-29   (today + recent max − elapsed)

  This is a rough estimate based on 5 recent features.
  It is not a probability forecast.
```

### Computation

All three values are based on the "recent 5" sample — the last 5 complete
features by completion timestamp.

| Value | Formula |
|---|---|
| Likely | `today + max(0, recent_median - elapsed)` |
| Optimistic | `today + max(0, recent_min - elapsed)` |
| Pessimistic | `today + max(0, recent_max - elapsed)` |

When elapsed exceeds the recent median/min/max, the projection clamps to today
(the feature is already overdue relative to that reference).

### For a phase

```
product forecast --phase 2 --naive

  Phase 2 — Products and IAM
  Features remaining: 5 (FT-009, FT-010, FT-011, FT-012, FT-013)

  Recent 5 complete features: median 4.01d, range 2.44 – 7.22d

  Naive projection assumes sequential work at recent pace:
    Likely completion:  2026-06-15   (today + 5 × median)
    Optimistic:         2026-06-06   (today + 5 × min)
    Pessimistic:        2026-07-02   (today + 5 × max)

  Assumes no parallelism and no dependency blocking.
  For a more precise forecast, export cycle times:
    product cycle-times --format csv > cycle-times.csv
```

Sequential multiplication is the simplest honest model for a phase. Real phase
timing involves parallelism and dependency ordering, which changes the numbers
in ways this tool doesn't model — the output says so explicitly.

### Flags

```bash
product forecast FT-015 --naive                   # single feature
product forecast --phase 2 --naive                # sequential phase projection
product forecast FT-015 --naive --sample-size 10  # use last 10 instead of 5
```

### Insufficient data

```
product forecast FT-015 --naive

  ✗ Insufficient data for naive projection.
  Only 2 features have both started and complete tags.
  Naive projection requires at least 3.

  View current cycle times:  product cycle-times
```

Minimum 3 features. Below that, the tool refuses to project — two data points
are not a sample.

---

## Integration with `product status`

`product status` gains a cycle time reference line for in-progress features:

```
product status

  Phase 1 — Cluster Foundation  [OPEN — exit criteria: 2/4 passing]
    FT-001  Cluster Foundation     complete      2.84d
    FT-002  mTLS Node Comms        complete      5.12d
    FT-003  Raft Consensus         in-progress   elapsed 3.2d  (recent median: 4.0d)
    FT-004  Block Storage          planned
```

Complete features show their cycle time. In-progress features show elapsed
plus the recent median for comparison. Planned features show nothing — there's
nothing honest to say yet.

The cycle time column is omitted entirely if fewer than 3 features have
completed (not enough data to be useful).

---

## `product.toml`

```toml
[cycle-times]
recent-window = 5              # how many recent features define "recent"
min-features = 3               # minimum complete features for projection
trend-threshold = 0.25         # ratio deviation for accelerating/slowing classification
```

---

## What This Replaces

The Monte Carlo forecasting spec is not built. The `started` tag (from
product-planning-due-date-spec.md) and the `product cycle-times --format csv`
export together give teams everything needed to run a Monte Carlo or any other
model in an external tool:

```bash
product cycle-times --format csv > cycle-times.csv
# Then run whatever model you want:
python scripts/monte_carlo.py cycle-times.csv
# Or import into a spreadsheet, a notebook, or a BI tool.
```

Product ships the visibility. Teams that want prediction ship the model.

This is the correct boundary. Cycle time visibility is universally useful and
scales from 3 features to 3000. Probabilistic forecasting is useful above 50
features — teams at that scale have real analytics stacks and will get better
results running their own models. Building a Monte Carlo into Product serves
neither end of the scale well.

---

## Session Tests

```
ST-320  cycle-times-lists-complete-features
ST-321  cycle-times-excludes-features-without-started-tag
ST-322  cycle-times-excludes-features-without-complete-tag
ST-323  cycle-times-uses-first-complete-tag-for-v2-features
ST-324  cycle-times-recent-5-computed-correctly
ST-325  cycle-times-trend-accelerating
ST-326  cycle-times-trend-stable
ST-327  cycle-times-trend-slowing
ST-328  cycle-times-in-progress-shows-elapsed
ST-329  cycle-times-json-valid-schema
ST-330  cycle-times-csv-parseable
ST-331  forecast-naive-single-feature
ST-332  forecast-naive-phase-sequential
ST-333  forecast-naive-insufficient-data
ST-334  forecast-naive-elapsed-exceeds-sample-clamps-to-today
ST-335  status-shows-cycle-time-column-when-data-present
ST-336  status-omits-cycle-time-column-when-below-min
```

---

## Invariants

- `product cycle-times` is a pure read operation. It never writes, never
  creates tags, never modifies front-matter.
- All values are derived exclusively from git tags. No estimates, no
  configuration, no manual data entry.
- Cycle time uses the first `complete` tag, not the most recent `complete-vN`.
  Re-verification reflects spec changes, not implementation time.
- `product forecast --naive` labels its output as "rough" and "not a
  probability forecast" in every invocation. The naming and output formatting
  make the limitations unambiguous.
- The `--format csv` output is stable and documented as the export format for
  external forecasting tools. Its schema does not change between Product
  versions without a schema-version bump.
- Below 3 complete features, both commands refuse to produce statistics.
  Product never extrapolates from two data points.
