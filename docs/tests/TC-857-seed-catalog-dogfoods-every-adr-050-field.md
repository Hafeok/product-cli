---
id: TC-857
title: seed_catalog_dogfoods_every_adr_050_field
type: invariant
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_857_seed_catalog_dogfoods_every_adr_050_field
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 0.2s
---

## Description

Schema-completeness dogfood test (invariant). After applying the
seed batch:

Assert that every front-matter field defined by ADR-050 is
exercised by at least one seed:

- `id`, `title`, `status` — all three seeds.
- `domains` — all three seeds (non-empty).
- `adrs` — all three seeds cite at least one ADR.
- `requires` — PAT-002 (non-empty).
- `examples` — all three seeds.
- `deprecated-by` — none of the three (status: live) — this
  field's absence is exercised across the seed set.

The invariant holds when the union of fields exercised by the
seeds equals (or contains) every required field in ADR-050.

## Formal specification

⟦Γ:Invariants⟧{
  every_required_pat_front_matter_field_is_present_in_at_least_one_seed
  the_optional_deprecated_by_field_is_correctly_absent_across_all_live_seeds
  the_seeds_collectively_demonstrate_the_schema_is_neither_over_broad_nor_missing_fields_needed_to_express_them
}

⟦Ε⟧⟨δ≜1.0;φ≜∞;τ≜◊⁺⟩