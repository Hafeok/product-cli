---
id: TC-648
title: cycle_times_uses_first_complete_tag_for_v2_features
type: invariant
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-036
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_648_cycle_times_uses_first_complete_tag_for_v2_features
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.6s
---

## TC — cycle-times uses the first `complete` tag, not the most recent `complete-vN` (invariant)

For every feature with multiple completion tags (`complete`,
`complete-v2`, `complete-v3`, ...), cycle time must be computed
against the timestamp of the *first* tag (`complete`). This
keeps cycle time stable across re-verification (ADR-036,
ADR-046 §2).

⟦Σ:Types⟧{
  Feat≜FeatureId;
  Tag≜(Feat, String, Instant);
  CycleTime≜(Feat, Instant, Instant, Days)
}
⟦Γ:Invariants⟧{
  ∀f:Feat with Tags(f)={t_started, t_complete, t_complete_v2, …}:
    cycle_time(f) = (instant(t_complete) - instant(t_started))
    ∧ cycle_time(f) ≠ (instant(t_complete_vN) - instant(t_started))  for any N≥2
}
⟦Λ:Scenario⟧{
  given≜FT-401 has product/FT-401/started @ 2026-04-08T13:00Z,
        product/FT-401/complete    @ 2026-04-11T09:14Z,
        product/FT-401/complete-v2 @ 2026-05-03T11:00Z (re-verify after spec change)
  when≜run(`product cycle-times`)
  then≜row for FT-401 shows cycle_time_days computed from
       complete@2026-04-11, not complete-v2@2026-05-03
}
⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩