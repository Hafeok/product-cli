---
id: ADR-024
title: Architectural Fitness Functions — Continuous Metric Tracking
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:37d76d04f58bb1fdda45d4d27828e911604a953fa1d4c17992bf851dc12af111
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
| `bundle_depth1_adr_p95` | 95th percentile of `depth-1-adrs` across all features | ↓ |
| `bundle_tokens_p95` | 95th percentile of `tokens-approx` across all features | ↓ |
| `bundle_domains_p95` | 95th percentile of `domains` count across all features | ↓ |
| `features_over_adr_threshold` | count of features where `depth-1-adrs` exceeds threshold | ↓ |

All metrics except `implementation_velocity`, `centrality_stability`, and the `features_over_*` count metrics are in [0.0, 1.0] or are raw counts/values. Bundle size metrics use percentile aggregation — per-feature bundle sizes are recorded in `metrics.jsonl` on each `product context --measure` call; the p95 values are recomputed by `product metrics record` from all available feature measurements.

---

### `metrics.jsonl`

Two entry types are appended to `metrics.jsonl`:

**Repository-wide snapshot** (written by `product metrics record`):
```json
{
  "type": "snapshot",
  "date": "2026-04-11T09:00:00Z",
  "commit": "abc123",
  "spec_coverage": 0.87,
  "test_coverage": 0.72,
  "exit_criteria_coverage": 0.61,
  "phi": 0.68,
  "gap_density": 0.03,
  "gap_resolution_rate": 0.75,
  "drift_density": 0.10,
  "centrality_stability": 0.02,
  "implementation_velocity": 2,
  "bundle_depth1_adr_p95": 6.0,
  "bundle_tokens_p95": 7800,
  "bundle_domains_p95": 4.0,
  "features_over_adr_threshold": 2
}
```

**Per-feature bundle measurement** (written by `product context FT-XXX --measure`):
```json
{
  "type": "bundle_measure",
  "date": "2026-04-11T09:14:22Z",
  "feature": "FT-003",
  "depth-1-adrs": 9,
  "depth-2-adrs": 14,
  "tcs": 12,
  "domains": 5,
  "tokens-approx": 11200
}
```

`metrics.jsonl` is committed to the repo. Merge conflicts are resolved by keeping both lines.

---

### Threshold Configuration

```toml
[metrics.thresholds]
spec_coverage           = { min = 0.90, severity = "error" }
test_coverage           = { min = 0.80, severity = "error" }
exit_criteria_coverage  = { min = 0.60, severity = "warning" }
phi                     = { min = 0.70, severity = "warning" }
gap_resolution_rate     = { min = 0.50, severity = "warning" }
drift_density           = { max = 0.20, severity = "warning" }

# Bundle size thresholds — signals features that may need splitting
bundle_depth1_adr_max   = { max = 8,    severity = "warning" }  # per-feature
bundle_tokens_max       = { max = 12000, severity = "warning" } # per-feature
bundle_domains_max      = { max = 6,    severity = "warning" }  # per-feature
features_over_adr_threshold = { max = 3, severity = "warning" } # repository-wide
```

Bundle size thresholds apply per-feature. When `product metrics threshold` runs, it checks every feature's last-measured `bundle` block against the per-feature thresholds and reports features that breach them. The `features_over_adr_threshold` metric is the repository-wide count of breaching features — this is what goes into CI as a gate.

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