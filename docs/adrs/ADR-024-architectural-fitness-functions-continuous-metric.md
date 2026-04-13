---
id: ADR-024
title: Architectural Fitness Functions — Continuous Metric Tracking
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** `product graph check` and `product gap check` provide point-in-time binary assessments: the graph is valid or it isn't, there are gaps or there aren't. They do not show trends. A repository where `phi` (formal block coverage) has been declining for six weeks is not distinguishable from one where it has been stable at 0.70 — both pass today's CI check. The decline is invisible until `phi` drops below the configured threshold.

Architectural fitness functions (from "Building Evolutionary Architectures") address this: define metrics that measure architectural properties, record them over time, and gate on both current values and trends.

**Decision:** `product metrics record` appends a JSON snapshot to `metrics.jsonl` on every merge to main. `product metrics threshold` checks current values against configured thresholds in CI. `product metrics trend` renders the time series. `metrics.jsonl` is committed to the repository — the history is version-controlled alongside the code it describes.

---

### Tracked Metrics

| Metric | Computation | Good direction |
|---|---|---|
| `spec_coverage` | features with ≥1 linked ADR / total features | ↑ |
| `test_coverage` | features with ≥1 linked TC / total features | ↑ |
| `exit_criteria_coverage` | features with exit-criteria TC / total features | ↑ |
| `phi` | mean formal block coverage across all invariant+chaos TCs | ↑ |
| `gap_density` | new gaps opened in last 7d / total ADRs | ↓ |
| `gap_resolution_rate` | gaps resolved / gaps opened, rolling 30d | ↑ |
| `drift_density` | unresolved drift findings / total ADRs | ↓ |
| `centrality_stability` | variance in top-5 ADR centrality ranks, week-over-week | ↓ |
| `implementation_velocity` | features moved to `complete` in last 7d | tracked |

All metrics except `implementation_velocity` and `centrality_stability` are in [0.0, 1.0]. `centrality_stability` is a variance value. `implementation_velocity` is a raw count.

---

### `metrics.jsonl`

One JSON object per line, appended on each `product metrics record` invocation:

```json
{"date":"2026-04-11T09:00:00Z","commit":"abc123","spec_coverage":0.87,"test_coverage":0.72,"exit_criteria_coverage":0.61,"phi":0.68,"gap_density":0.03,"gap_resolution_rate":0.75,"drift_density":0.10,"centrality_stability":0.02,"implementation_velocity":2}
```

`metrics.jsonl` is committed to the repo. The history is inspectable with `git log -p metrics.jsonl`. Merge conflicts on `metrics.jsonl` are resolved by keeping both lines — the file is append-only and line order does not matter for trend computation.

---

### Threshold Configuration

```toml
[metrics.thresholds]
spec_coverage = { min = 0.90, severity = "error" }
test_coverage = { min = 0.80, severity = "error" }
exit_criteria_coverage = { min = 0.60, severity = "warning" }
phi = { min = 0.70, severity = "warning" }
gap_resolution_rate = { min = 0.50, severity = "warning" }
drift_density = { max = 0.20, severity = "warning" }
```

`product metrics threshold` exits 1 if any `error`-severity threshold is breached, exits 2 if any `warning`-severity threshold is breached. This integrates with the existing exit code model (ADR-009).

---

### `product metrics trend` Output

ASCII sparkline for quick terminal inspection:

```
product metrics trend --metric phi --last 30d

phi (formal block coverage) — last 30 days
0.80 ┤                                    ╭──
0.75 ┤                               ╭───╯
0.70 ┤ ──────────────────────────────╯      ← threshold: 0.70
0.65 ┤
     └────────────────────────────────────
     2026-03-12              2026-04-11

current: 0.78  Δ7d: +0.03  Δ30d: +0.12  trend: ↑
```

`product metrics trend` with no flags shows all metrics as a summary table with current value, 7-day delta, and trend arrow.

---

**Rationale:**
- Committing `metrics.jsonl` to the repository is the correct storage decision. It co-locates the metric history with the artifacts it measures, it is version-controlled, it requires no external service, and it is inspectable with standard git tooling. The alternative (a metrics database or external dashboard) adds operational dependencies that contradict Product's repository-native design principle.
- ASCII sparklines in terminal are sufficient for a developer tool. An external dashboard would provide more visual richness but would require a server, a URL, and a login. The terminal is always available, especially during the authoring sessions where metrics are most relevant.
- `implementation_velocity` is tracked but has no threshold. It is an informational metric — fast velocity is not always good (quality may be declining), slow velocity is not always bad (hard problems take time). It should be observed, not gated on.
- Appending to `metrics.jsonl` rather than updating a single record means the full history is always available without a database. Trend computation reads all records at query time — acceptable for a file that grows by one line per merge to main.