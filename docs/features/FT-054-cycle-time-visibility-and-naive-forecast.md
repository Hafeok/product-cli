---
id: FT-054
title: Cycle Time Visibility and Naive Forecast
phase: 5
status: planned
depends-on:
- FT-036
- FT-037
- FT-053
adrs:
- ADR-036
- ADR-045
- ADR-046
tests:
- TC-645
- TC-646
- TC-647
- TC-648
- TC-649
- TC-650
- TC-651
- TC-652
- TC-653
- TC-654
- TC-655
- TC-656
- TC-657
- TC-658
- TC-659
- TC-660
- TC-661
- TC-662
- TC-663
- TC-664
domains:
- observability
- scheduling
domains-acknowledged:
  ADR-038: product cycle-times and product forecast --naive are read-only commands — no front-matter mutations, no tag writes, no request-log entries. They never interact with the request pipeline and therefore do not need new request-shape extensions.
  ADR-042: Uses existing TC types — scenario for behavioural rows, invariant for the JSON/CSV schema stability, first-complete-tag and clamp-to-today rules, and exit-criteria for the consolidated check-list. No new TC types introduced; ADR-042's reserved-structural / open-descriptive partition is unchanged.
  ADR-043: 'Implementation follows the slice + adapter pattern: new src/cycle_times/ slice exposes pure build_* and render_* functions, thin src/commands/cycle_times.rs and src/commands/forecast.rs adapters load the graph, call the slice, and wrap results in Output::text or Output::both. No monolithic handlers.'
  ADR-040: No new verify stage or LLM-boundary hook. product cycle-times and product forecast --naive are pure read commands with their own render path; they do not extend the verify pipeline or the semantic-analysis bundle surface.
  ADR-041: No absence TCs or ADR removes/deprecates interaction — cycle-times and forecast --naive are additive read surfaces over existing git tags. Nothing is removed or deprecated by this feature.
---

## Description

FT-053 and ADR-045 delivered the two anchors needed to compute
cycle time: the `product/FT-XXX/started` tag (created on the
first `planned → in-progress` transition) and the existing
`product/FT-XXX/complete` tag (ADR-036). With both in place the
graph contains enough signal to answer "how long do features
take, and is that pace changing?"

This feature ships `product cycle-times` (a read-only view of
historical cycle time derived from git tags), `product forecast
--naive` (a rough projection deliberately labelled as rough),
and a cycle-time column on `product status` for features that
already have data. It explicitly does not ship a probabilistic
model — ADR-046 pins that decision and explains why.

The full spec is
[`docs/product-cycle-times-spec.md`](/docs/product-cycle-times-spec%20%282%29.md);
the pinned decisions live in ADR-046.

---

## Depends on

- **FT-036** — tag-based implementation tracking; `complete` tag
  is the end-anchor for every cycle-time computation and the
  first-tag-authority rule lives in ADR-036's namespace.
- **FT-037** — drift detection / tag listing; `product
  cycle-times` reuses the git-tag plumbing (`git for-each-ref`,
  `git log -1 --format=%aI`) exercised by the existing tags
  surface.
- **FT-053** — planning / started tag; without
  `product/FT-XXX/started` there is no start-anchor. This
  feature does not create the tag, only consumes it.

---

## Scope of this feature

### In

1. **`product cycle-times` command.** Read-only. Lists every
   feature with both `started` and `complete` tags. Columns:
   feature, started (YYYY-MM-DD), completed (YYYY-MM-DD), cycle
   time (days, one decimal). Summary footer: count, recent-N
   stats (median / min / max), all-time stats (median / min /
   max), trend classifier.
2. **`--recent N`, `--phase N`, `--in-progress`, `--format
   {text|json|csv}` flags.** Text is the default, JSON and CSV
   are stable export interfaces (ADR-046 §10). `--in-progress`
   replaces the complete-features table with an elapsed-so-far
   table plus a reference median.
3. **`product forecast --naive` command.** Single feature mode
   (`product forecast FT-XXX --naive`) projects likely /
   optimistic / pessimistic completion dates using the recent-N
   sample. Phase mode (`product forecast --phase N --naive`)
   multiplies K remaining features by recent median / min / max.
   Every invocation labels the output as rough and points at the
   CSV export for better models.
4. **`--sample-size N` flag on `product forecast`.** Overrides
   `[cycle-times].recent-window` for a single invocation.
5. **`product status` cycle-time column.** Complete features
   render their cycle time; in-progress features render
   elapsed-so-far plus the recent median for comparison; planned
   features render nothing. Column is omitted entirely if fewer
   than `[cycle-times].min-features` features have both tags.
6. **`[cycle-times]` config section.** Three keys: `recent-window`
   (default 5), `min-features` (default 3), `trend-threshold`
   (default 0.25). Validated at load time.
7. **Insufficient-data refusal.** Below `min-features` complete
   features, `cycle-times` returns an empty result (exit 0) and
   `forecast --naive` returns an explanatory message with exit
   code 2. Neither command extrapolates from two data points.
8. **Unit + integration tests** covering tag parsing, the
   recent-5 computation, the three trend classifications, the
   in-progress elapsed rendering, JSON / CSV schema stability,
   the insufficient-data guards, the clamp-to-today behaviour,
   and the status-column presence / absence rules.

### Out

- **Probabilistic forecasting.** No Monte Carlo, no P50/P80/P95,
  no regression or fitted distribution. ADR-046 pins this.
- **Stored cycle time in front-matter.** Cycle time is derived
  from tag timestamps at read time. No new feature fields.
- **Bucketing by size / complexity / team.** Real forecasting
  work happens in the team's analytics stack via the CSV
  export; this feature ships the export, not the model.
- **`product forecast` without `--naive`.** The flag is
  mandatory — the UX contract is that users opt into a
  projection they know is rough. No unlabelled forecast
  surface exists.
- **Mean / stddev / IQR / percentiles beyond min/max/median.**
  Statistics that imply distribution precision the sample does
  not support are excluded.
- **Direct BI or SQLite export.** CSV plus JSON cover the
  external-tool handoff; richer formats are deferred.

---

## Commands

All new surfaces are additive; no existing commands change
shape.

```bash
product cycle-times                          # default text table
product cycle-times --recent N               # last N completed features
product cycle-times --phase 1                # scope to phase 1 features
product cycle-times --in-progress            # elapsed-so-far table
product cycle-times --format json            # machine-readable
product cycle-times --format csv             # spreadsheet / external tools

product forecast FT-015 --naive              # single-feature projection
product forecast --phase 2 --naive           # sequential phase projection
product forecast FT-015 --naive --sample-size 10   # override recent-window

product status                               # now includes cycle-time column when ≥ min-features complete
```

---

## Implementation notes

- **`src/cycle_times/` (new slice).** Pure `build_*` functions
  take `(graph, git_tag_reader, now, config)` and return a
  `CycleTimeReport` struct. `render_*` functions produce text /
  JSON / CSV strings. No direct I/O in the slice.
- **`src/commands/cycle_times.rs` + `commands/forecast.rs`.**
  Thin adapters that load the graph, pull tag timestamps via a
  shared `git_tags::read_timestamps` helper, call the slice, and
  wrap the result in `Output::text { ... }` or `Output::both {
  text, json }`. Follow ADR-043 slice + adapter pattern.
- **`src/git_tags/` — new `read_all_started_and_complete`
  helper.** Batched `git for-each-ref refs/tags/product/*` to
  avoid per-feature shell-outs on large repos. Returns
  `HashMap<FeatureId, (Option<DateTime>, Option<DateTime>)>`.
  Benchmarks in `benches/graph_bench.rs` cover the 500-feature
  case.
- **First-complete-tag rule (ADR-046 §2).** The helper must
  return the earliest `product/FT-XXX/complete` timestamp, not
  the most recent `complete-vN`. Implement by sorting matching
  tags lexicographically — `complete` sorts before `complete-v2`
  and the tag timestamp for `complete` is the authoritative
  first verification.
- **Trend classifier (ADR-046 §4).** Pure function:
  `(recent_median, all_median, threshold) -> Trend`. Ratio
  `(recent - all) / all` compared against `±threshold`. Below 6
  complete features, return `None` — the caller renders no
  trend line.
- **Naive projection (ADR-046 §6).** Pure function: `(today,
  elapsed, recent_min, recent_median, recent_max) -> (likely,
  optimistic, pessimistic)`. `max(0, …)` clamp matches the
  elapsed-exceeds-sample invariant (ADR-046 §9).
- **Phase projection (ADR-046 §7).** `(today, k_remaining,
  recent_min, recent_median, recent_max) -> (likely, optimistic,
  pessimistic)`. Identical shape, different input — reuses the
  same rendering surface.
- **`src/config.rs`.** New `[cycle-times]` section with three
  keys. Defaults live in `config.rs`; `product.toml` parsing
  validates `trend-threshold` in `[0.0, 1.0]` and refuses
  negative `min-features` / `recent-window`.
- **`src/status/` — cycle-time column.** Extend the existing
  status rendering to emit a column when `complete_count >=
  min_features`. Complete features render `Nd`; in-progress
  features render `elapsed Nd (recent median: Md)`; planned
  features render an empty cell. Matches the FT-053 due-date
  column conventions.
- **JSON / CSV stability.** Serde-derived schemas, locked by
  TCs ST-329 and ST-330. Any field addition requires a
  schema-version bump (ADR-046 §10). Document the contract in
  `docs/guide/FT-054-*.md`.
- **File-length budget.** Every touched file must stay under
  400 lines. Plan to split `src/cycle_times/` into `model.rs`
  (pure types), `compute.rs` (stats), `render.rs` (text + CSV),
  and `json.rs` (serde schemas) from the start.
- **Dependencies.** `chrono` and `csv` are already direct
  dependencies; no new crates required.

---

## Acceptance criteria

A developer running on a populated test fixture can:

1. Run `product cycle-times` on a repo with 14 complete
   features and observe the text table, the count, the recent-5
   and all-time summary rows, and the trend indicator.
2. Run `product cycle-times --format csv` and parse the output
   with the `csv` crate — header row exactly matches
   `feature_id,started,completed,cycle_time_days,phase`, every
   subsequent row has valid ISO 8601 timestamps and a numeric
   cycle time with one decimal.
3. Run `product cycle-times --format json` and deserialise the
   output — `features[]` has the documented shape, `summary`
   contains `count`, `recent_5`, `all`, and `trend` with the
   expected field names.
4. Run `product cycle-times --in-progress` on a fixture with
   both in-progress and complete features — output shows only
   the in-progress features with elapsed-so-far, plus the
   recent-5 median reference line.
5. Apply a fixture where the recent-5 median is 25%+ below the
   all-time median and observe `Trend: accelerating`; same with
   25%+ above → `slowing`; within ±25% → `stable`; fewer than
   6 complete features → no trend line at all.
6. Run `product forecast FT-015 --naive` on an in-progress
   feature with `elapsed = 2.3d` and recent stats of median 4.01d
   / min 2.44d / max 7.22d — observe likely, optimistic,
   pessimistic dates computed per ADR-046 §6, plus the "rough
   estimate / not a probability forecast" labels.
7. Run `product forecast --phase 2 --naive` with 5 remaining
   features — observe likely = `today + 5 * median`, optimistic
   = `today + 5 * min`, pessimistic = `today + 5 * max`, plus
   the parallelism / CSV export callouts.
8. Run `product forecast FT-XXX --naive` on a fixture with only
   2 complete features — observe the insufficient-data message
   and exit code 2, no projection rendered.
9. Run `product forecast FT-XXX --naive` on a feature whose
   elapsed time exceeds the recent maximum — observe all three
   projections clamp to today (never a past date).
10. Run `product status` with ≥ 3 complete features — observe
    the cycle-time column for complete features, elapsed plus
    recent median for in-progress features, empty cell for
    planned features. Drop below 3 complete — column is omitted
    entirely.
11. Run `cargo test`, `cargo clippy -- -D warnings -D
    clippy::unwrap_used`, and `cargo build` and observe all pass.

See the exit-criteria TC for the consolidated check-list.

---

## Follow-on work

- **Probabilistic forecasting in an external tool.** The CSV
  export is the hand-off point. `scripts/monte_carlo.py` (not
  part of this feature) demonstrates the external-model pattern.
- **Bucketing by feature domain / size.** Once teams have
  enough features and a labelling scheme, bucketed statistics
  could move into Product. Deferred — the CSV export covers
  this for now.
- **Forecast refinement based on in-flight observations.** A
  Bayesian update on the naive projection as elapsed time grows
  would be nice; not enough data at current scale. Deferred.
- **Per-phase burndown chart.** Pure presentation over the
  existing data. Deferred until demand exists.
