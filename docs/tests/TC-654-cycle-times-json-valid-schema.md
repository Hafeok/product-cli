---
id: TC-654
title: cycle_times_json_valid_schema
type: invariant
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_654_cycle_times_json_valid_schema
---

## TC — `product cycle-times --format json` matches a documented, versioned schema (invariant)

The JSON output of `product cycle-times --format json` is a
stable external interface for teams running their own
forecasting models. Field names, shape, and types must not drift
between Product versions without a documented schema bump
(ADR-046 §10).

⟦Σ:Types⟧{
  Report≜{
    features: [FeatureRow],
    summary: { count: int, recent_5: Stats, all: Stats, trend: TrendOpt }
  };
  FeatureRow≜{ id: String, started: ISO8601, completed: ISO8601,
               cycle_time_days: Float, phase: Int };
  Stats≜{ median: Float, min: Float, max: Float };
  TrendOpt≜"accelerating" | "stable" | "slowing" | null
}
⟦Γ:Invariants⟧{
  ∀fixture f: deserialize(run(`product cycle-times --format json`, f), Report) ⇒ ok
  ∧ forall row ∈ report.features:
      isoparse(row.started).ok ∧ isoparse(row.completed).ok
      ∧ row.cycle_time_days ≥ 0
      ∧ row.cycle_time_days == round1(row.cycle_time_days)
  ∧ report.summary.trend ∈ {"accelerating","stable","slowing",null}
  ∧ (report.summary.count < 6 ⇒ report.summary.trend = null)
}
⟦Λ:Scenario⟧{
  given≜fixture with 14 complete features (as in spec)
  when≜run(`product cycle-times --format json`)
  then≜output deserialises into Report exactly; any additional
        top-level or per-row keys cause the test to fail; any
        missing required key causes the test to fail
}
⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
