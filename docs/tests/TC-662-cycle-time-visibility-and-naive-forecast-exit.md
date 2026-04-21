---
id: TC-662
title: cycle_time_visibility_and_naive_forecast_exit
type: exit-criteria
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_662_cycle_time_visibility_and_naive_forecast_exit
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.5s
---

## Exit Criteria — FT-054 Cycle Time Visibility and Naive Forecast

FT-054 is complete when all of the following hold:

1. **`product cycle-times` ships.** Lists every feature with
   both `product/FT-XXX/started` and `product/FT-XXX/complete`
   tags, rendering feature id, started date, completed date,
   and cycle time in days with one decimal. Summary footer
   reports count, recent-N stats, all-time stats, and (if
   count ≥ 6) a trend classifier.
2. **Flags work as specified.** `--recent N`, `--phase N`,
   `--in-progress`, `--format {text,json,csv}` all produce the
   documented output. `text` is default.
3. **First `complete` tag wins.** Features with `complete-vN`
   tags compute cycle time from the first `complete` tag
   timestamp, not the most recent version (ADR-046 §2).
4. **Trend classifier is three-state.** `accelerating` when
   recent-N median is > 25% below all-time median; `stable`
   within ±25%; `slowing` when > 25% above. Below 6 complete
   features, trend line is omitted. Threshold configurable via
   `[cycle-times].trend-threshold` (default 0.25).
5. **`product forecast --naive` ships.** Single-feature mode
   produces likely / optimistic / pessimistic dates via
   `today + max(0, recent_{median,min,max} - elapsed)`. Phase
   mode multiplies by `K` remaining features. Every invocation
   labels output as rough and not a probability forecast.
6. **Insufficient-data refusal.** Below `[cycle-times].min-features`
   (default 3) complete features, `cycle-times` returns an empty
   result (exit 0) and `forecast --naive` returns exit 2 with an
   explanatory message. No extrapolation.
7. **Clamp invariant holds.** When elapsed exceeds the recent
   sample, the corresponding projection clamps to today. No
   past dates are ever rendered as future completion estimates.
8. **JSON and CSV schemas are stable.** The documented shapes in
   `docs/product-cycle-times-spec.md` are enforced by
   invariant TCs and must not change without a schema-version
   bump.
9. **`product status` gains a cycle-time column** when
   `complete_count >= min_features`; complete features show
   their cycle time, in-progress features show
   `elapsed Nd (recent median: Md)`, planned features show
   nothing. Column is omitted entirely below the threshold.
10. **`product cycle-times` is read-only.** No tag writes, no
    front-matter mutations, no request-log entries. Only git
    tag reads.
11. **No probabilistic forecast surface.** No Monte Carlo,
    percentile fits, mean / stddev, or unlabelled forecast
    commands exist anywhere in the CLI.
12. **`cargo test`, `cargo clippy -- -D warnings -D
    clippy::unwrap_used`, and `cargo build`** all pass.
13. **Every TC under FT-054 has `runner: cargo-test` and
    `runner-args` set** to the matching integration test
    function name.