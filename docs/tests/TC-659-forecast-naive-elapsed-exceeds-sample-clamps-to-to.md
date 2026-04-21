---
id: TC-659
title: forecast_naive_elapsed_exceeds_sample_clamps_to_today
type: invariant
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_659_forecast_naive_elapsed_exceeds_sample_clamps_to_today
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.5s
---

## TC — elapsed exceeds recent sample ⇒ projection clamps to today (invariant)

When an in-progress feature's elapsed time already exceeds the
recent median / min / max, the corresponding projection clamps
to today. The naive forecast must never output a past date as a
future completion estimate (ADR-046 §9).

⟦Σ:Types⟧{
  Today≜Date;
  Elapsed≜Float;
  Recent≜{ min: Float, median: Float, max: Float };
  Proj≜{ likely: Date, optimistic: Date, pessimistic: Date }
}
⟦Γ:Invariants⟧{
  ∀(today, elapsed, recent):
    let p = project_naive(today, elapsed, recent) in
      p.likely ≥ today ∧ p.optimistic ≥ today ∧ p.pessimistic ≥ today
  ∧ elapsed ≥ recent.max ⇒ p.pessimistic = today
  ∧ elapsed ≥ recent.median ⇒ p.likely = today
  ∧ elapsed ≥ recent.min ⇒ p.optimistic = today
  ∧ project_naive uses formula
      date_add(today, max(0, recent.X - elapsed))
      for X ∈ {min, median, max}
}
⟦Λ:Scenario⟧{
  given≜today=2026-06-10, elapsed=10.0d,
        recent = { min: 2.44, median: 4.01, max: 7.22 }
  when≜project_naive(today, elapsed, recent)
  then≜likely = optimistic = pessimistic = today (all three
       clamp because elapsed > recent.max)
}
⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩