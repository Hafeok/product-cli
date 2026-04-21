---
id: ADR-046
title: Cycle Time Visibility — Descriptive Statistics Over Probabilistic Forecasting
status: accepted
features: []
supersedes: []
superseded-by: []
domains:
- observability
- scheduling
scope: feature-specific
content-hash: sha256:f9204281e1f959467806cf3fc3c8a9efbffe7265e2365107805a395561f4c0d3
---

**Status:** Proposed

**Context:** FT-053 and ADR-045 add the `product/FT-XXX/started`
tag and the `due-date` feature field, the two anchors needed to
compute cycle time. With both `started` and `complete` tags in
place, the question "how long do features take, and is that
pace changing?" becomes answerable from the graph.

A natural follow-on is probabilistic forecasting: fit a
distribution to observed cycle times, run a Monte Carlo, report
P50/P80/P95 completion windows for a feature or a phase. The
`docs/product-forecasting-spec.md` draft sketched exactly that.

This ADR rejects the Monte Carlo direction and pins a narrower
scope: ship cycle-time **visibility**, not cycle-time
**prediction**. The reasoning matters because the difference
between the two is a design choice about what Product claims to
know, not a missing feature.

**The statistical problem.** Product repositories at the scale
where Product is most useful (single-team, single-project) have
tens of complete features, not hundreds. A Monte Carlo fit to
14 data points produces a posterior predictive distribution
whose 95% interval is wider than the point estimate itself. At
30 features the interval is still wide enough that reporting
P50/P80/P95 overstates the precision of the estimate. The
numbers look authoritative; they are not.

**The boundary problem.** Teams that outgrow descriptive
statistics (50+ complete features, multiple parallel work
streams, historical bucketing by size or team) want bespoke
models — correlation analysis, complexity stratification,
feature-size regression — that a general-purpose Monte Carlo in
Product cannot provide. The useful forecasting work happens in
the team's analytics stack with its own cycle-time export,
notebooks, and BI tooling. Product's contribution at that scale
is a clean, stable CSV export, not another probabilistic engine.

**Decision:** `product cycle-times` surfaces historical cycle
times as descriptive statistics (median, range, trend) derived
from git tags. `product forecast --naive` produces a rough
projection labelled explicitly as rough. No probabilistic model,
no confidence intervals, no percentile fits. The CSV export is
a stable, versioned interface for teams running their own models.

### Decisions pinned by this ADR

1. **Cycle time is the elapsed duration from
   `product/FT-XXX/started` to `product/FT-XXX/complete`, in
   days with one decimal.** Both tag timestamps are ISO 8601
   instants (`git log -1 --format=%aI`). Features missing either
   tag are excluded from cycle-time output.
2. **The authoritative `complete` tag is the first one, not the
   most recent `complete-vN`.** Re-verification (ADR-036)
   reflects spec updates, not implementation time. Using the
   earliest `complete` tag gives cycle time a stable meaning:
   "how long did the feature take to first pass verification."
3. **Descriptive statistics only.** `product cycle-times`
   reports count, median, min, max, and a trend classifier. No
   mean (outlier-sensitive), no standard deviation (meaningless
   on a right-skewed sample of this size), no percentiles
   beyond min/max/median.
4. **Trend classifier is a three-state comparison, not a
   regression.** Compare the median of the most recent N (default
   5) complete features to the median of all completed features.
   Ratio within ±25% → `stable`; recent is more than 25% below
   all-time → `accelerating`; recent is more than 25% above
   all-time → `slowing`. Below 6 complete features, the trend
   is omitted (insufficient history to distinguish recent from
   all-time).
5. **`product forecast --naive` is labelled naive in name and
   output.** Every invocation renders "rough estimate" and
   "not a probability forecast" in plain text. The word `naive`
   is not optional — there is no `--accurate` flag, no
   `--monte-carlo` mode, no unlabelled forecast surface.
6. **Naive projection uses the recent-N sample only.** Likely =
   `today + max(0, recent_median - elapsed)`. Optimistic =
   `today + max(0, recent_min - elapsed)`. Pessimistic =
   `today + max(0, recent_max - elapsed)`. The sample window N
   is configurable via `[cycle-times].recent-window` (default 5)
   and overrideable at invocation via `--sample-size`.
7. **Phase projection is sequential multiplication.** For a
   phase with K remaining features, likely = `today + K *
   recent_median`. Same for min/max. Output explicitly states
   the assumption ("assumes no parallelism and no dependency
   blocking") and points the user at the CSV export for a
   better model.
8. **Minimum sample size is 3 complete features.** Below 3,
   both `cycle-times` and `forecast --naive` refuse to report
   statistics. "Insufficient data" with exit code 0 for
   `cycle-times` (empty result), exit code 2 for
   `forecast --naive` (user asked for a projection we cannot
   responsibly give). Configurable via `[cycle-times].min-features`.
9. **Elapsed-exceeds-sample clamps to today.** When a
   feature's elapsed time already exceeds the recent min /
   median / max, the corresponding projection clamps to today
   (`max(0, …)`). The output surface never shows a past date as
   a future completion estimate.
10. **`--format json` and `--format csv` are stable export
    interfaces.** Their schemas do not change between Product
    versions without a documented schema bump. CSV columns:
    `feature_id, started, completed, cycle_time_days, phase`.
    Timestamps are ISO 8601; cycle time is a decimal number
    (one fractional digit). This is the contract for external
    forecasting tools.
11. **`product cycle-times` is read-only.** No tag writes, no
    front-matter mutations, no request-log entries. All values
    are derived from git tags at invocation time (matches
    ADR-003's derived-graph posture).
12. **`product status` gains a cycle-time column when
    data-sufficient.** Complete features render their cycle
    time; in-progress features render `elapsed Nd (recent
    median: Md)`; planned features render nothing. If fewer
    than `min-features` complete features exist, the column is
    omitted entirely rather than showing partial data.
13. **`product/FT-XXX/started` is required in-progress.** The
    cycle-time computation relies on it; FT-053 creates it on
    `planned → in-progress` transitions. In-progress features
    without a `started` tag are reported as unreachable by
    `--in-progress` and skipped from the "Elapsed" column with
    a warning.

### Scope — what is in and what is explicitly not

| Scope | In | Out |
|---|---|---|
| Command surface | `product cycle-times`, `product forecast --naive`, `product status` cycle-time column | `product forecast` without `--naive`; any Monte Carlo, probabilistic, or percentile-based forecast |
| Statistics | count, median, min, max, trend (3-state) | mean, stddev, percentiles beyond min/max/median, confidence intervals |
| Data source | `product/FT-XXX/started` + `product/FT-XXX/complete` git tags | stored `started-at`/`completed-at` front-matter fields, manual date entry, external trackers |
| Export | JSON, CSV | Parquet, SQLite, direct BI connectors |
| Projection inputs | recent-N sample (default 5), min-features (default 3), trend threshold (default 0.25) | sizing buckets, feature-complexity weights, team-velocity adjustments |
| Config | `[cycle-times]` section with `recent-window`, `min-features`, `trend-threshold` | `[forecast]` section — kept minimal on purpose |

**Rationale:**

- **Scale-appropriate.** The teams Product serves today do not
  have enough features for a probabilistic model to produce
  meaningful intervals. Shipping one anyway would generate
  output that looks precise and isn't.
- **Honest labelling.** Naming the command `forecast --naive`
  and annotating every output with "rough estimate / not a
  probability forecast" makes the limitation part of the UX,
  not a footnote.
- **Clean export boundary.** The CSV schema is the stable
  contract for teams that want real forecasting. Product ships
  the visibility; teams that want prediction ship the model in
  their own stack. This is the correct separation of concerns.
- **First-tag authority.** Using the first `complete` tag
  (rather than `complete-vN`) makes cycle time reflect
  implementation time, not re-verification churn. Consistent
  with ADR-036's re-verification semantics.
- **Trend as direction, not prediction.** The three-state
  `accelerating / stable / slowing` classifier communicates
  direction without committing to a rate. A regression line
  with a confidence band would overstate the signal.
- **Sequential phase projection.** Real phase timing depends on
  parallelism and dependency ordering. Multiplication is the
  simplest model that is demonstrably wrong in a known
  direction — the output says so, and the CSV export is
  pointed to for anything better.
- **Elapsed-exceeds clamp.** A naive projection that reports a
  past date would be user-hostile; clamping to today is the
  smallest correct behaviour.

**Rejected alternatives:**

- **Ship a Monte Carlo.** Rejected. At <50 features the
  intervals are wider than the point estimate; at >50 features
  the team already has a better tool. Serves neither scale
  well.
- **Percentile output (P50, P80, P95).** Rejected. Percentiles
  on a sample of 14 are non-robust to a single outlier and
  imply precision the data does not have. Min/max are the
  honest bounds.
- **Stored `cycle-time-days` in front-matter.** Rejected. Same
  rename/rebase problem ADR-036 identified for commit SHAs and
  ADR-045 identified for `started-at`. Derive from tags at
  read time.
- **`product forecast` without the `--naive` flag required.**
  Rejected. The flag is a UX contract: users must opt into a
  projection they know is rough. Removing it signals authority
  the output does not earn.
- **Configurable trend classifier (four-state, five-state,
  numeric).** Rejected. Three states cover the practically
  useful directions. Extra granularity encourages reading
  signal into noise.
- **Stddev or IQR in summary output.** Rejected. The sample is
  too small for these to be meaningful and their presence
  implies "you can trust this distribution."
- **Ship the forecasting model alongside visibility in one
  feature.** Rejected. Visibility is a narrow, self-contained
  capability with a stable schema. Forecasting is a separate
  scope that should be evaluated on its own merits and
  deferred until the data supports it.
- **Skip the `--in-progress` surface.** Rejected. Showing
  elapsed-so-far alongside the recent median gives users the
  most useful question ("is this one taking longer than
  usual?") without the tool making a prediction.

### Test coverage

| Decision | Covered by TC (title) |
|---|---|
| Lists complete features only | `cycle-times-lists-complete-features` |
| Excludes no-started | `cycle-times-excludes-features-without-started-tag` |
| Excludes no-complete | `cycle-times-excludes-features-without-complete-tag` |
| Uses first complete tag | `cycle-times-uses-first-complete-tag-for-v2-features` |
| Recent-5 computation | `cycle-times-recent-5-computed-correctly` |
| Trend — accelerating | `cycle-times-trend-accelerating` |
| Trend — stable | `cycle-times-trend-stable` |
| Trend — slowing | `cycle-times-trend-slowing` |
| In-progress elapsed | `cycle-times-in-progress-shows-elapsed` |
| JSON schema stable | `cycle-times-json-valid-schema` |
| CSV schema stable | `cycle-times-csv-parseable` |
| Naive single feature | `forecast-naive-single-feature` |
| Naive phase sequential | `forecast-naive-phase-sequential` |
| Insufficient data guard | `forecast-naive-insufficient-data` |
| Elapsed-clamps-to-today | `forecast-naive-elapsed-exceeds-sample-clamps-to-today` |
| Status column present | `status-shows-cycle-time-column-when-data-present` |
| Status column absent | `status-omits-cycle-time-column-when-below-min` |
