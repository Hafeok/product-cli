---
id: TC-732
title: product_schema and request validator share field allowlist
type: invariant
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-038
phase: 5
runner: cargo-test
runner-args: tc_732_product_schema_and_request_validator_share_field_allowlist
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.5s
---

## Why this TC exists

ADR-038 / FT-062 promise that the `product_schema` MCP tool and the
mutation-field validator (E026) consult the **same** source of truth for
recognised front-matter field names. If the two could drift, an agent could
ask `product_schema feature` for the field list, send a request using a
returned name, and have the request rejected — exactly the trust-eroding
failure mode this feature exists to prevent.

This TC pins the parity invariant by asserting that `field_schema` is the
single canonical source: every name the request validator accepts as a
known mutation field is the same set the public schema constants expose,
and the constants are the basis of both surfaces.

## Invariant

⟦Γ:Invariants⟧{
  ∀a:ArtifactType: known_fields_for(a) = FIELDS_CONST(a)
  ∀a:ArtifactType, ∀f ∈ known_fields_for(a):
    is_known_field(a, f) = true
  is_known_field(_, "body") = true
}

## Steps

1. Call `product_lib::field_schema::FEATURE_FIELDS` and
   `product_lib::field_schema::known_fields_for(ArtifactType::Feature)`;
   assert they refer to the same slice (pointer-equal `&'static`).
2. Repeat for ADR, TC, DEP.
3. For every field in each constant, assert
   `field_schema::is_known_field(at, field) == true`.
4. Assert `is_known_field(_, "body") == true` for every artifact type.

## Notes

This is a structural fitness check — the implementation guarantees the
invariant by construction (the slice and the lookup function return the
same `&'static`). The test exists to catch a future refactor that
accidentally introduces a second source of truth.