---
id: TC-655
title: cycle_times_csv_parseable
type: invariant
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_655_cycle_times_csv_parseable
---

## TC — `product cycle-times --format csv` is parseable and schema-stable (invariant)

The CSV output is the documented export format for external
forecasting tools. Header and per-row shape must be stable
across Product versions (ADR-046 §10).

⟦Σ:Types⟧{
  Header≜["feature_id","started","completed","cycle_time_days","phase"];
  Row≜(String, ISO8601, ISO8601, OneDecimal, Int)
}
⟦Γ:Invariants⟧{
  ∀fixture f with ≥1 complete feature:
    let csv = run(`product cycle-times --format csv`, f) in
      parse_csv(csv).header == Header
      ∧ ∀row ∈ parse_csv(csv).rows:
          matches_row_shape(row)
          ∧ isoparse(row.started).ok
          ∧ isoparse(row.completed).ok
          ∧ row.cycle_time_days_string =~ /^[0-9]+\.[0-9]$/
  ∧ ∀fixture f with 0 complete features:
      run(`product cycle-times --format csv`, f) == header_only_line
}
⟦Λ:Scenario⟧{
  given≜fixture with 14 complete features
  when≜run(`product cycle-times --format csv`) piped through
       the `csv` crate's Reader with has_headers(true)
  then≜14 records deserialise successfully; each record's
       started and completed parse via DateTime::parse_from_rfc3339;
       cycle_time_days parses as f64 and its formatted string has
       exactly one fractional digit
}
⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
