---
id: TC-491
title: request mutation ops cover set append remove delete with dot-notation
type: scenario
status: passing
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_491_request_mutation_ops_cover_set_append_remove_delete_with_dot_notation
last-run: 2026-04-17T11:51:10.501467904+00:00
last-run-duration: 0.2s
---

Validates FT-041 / ADR-038 decision 4.

**Setup:** fixture with an existing feature FT-X that has `adrs: [ADR-A, ADR-B]`, `domains: [api]`, `domains-acknowledged: { api: "existing reason" }`, and an existing DEP-Z with `interface: { port: 1234 }`.

**Act:** apply a `type: change` request that exercises all four ops, including dot-notation:
- `set field: phase value: 3` on FT-X (scalar set)
- `append field: adrs value: ADR-C` on FT-X (array append)
- `remove field: adrs value: ADR-A` on FT-X (array remove)
- `delete field: domains-acknowledged.api` on FT-X (optional nested delete)
- `set field: domains-acknowledged.security value: "no trust boundary"` on FT-X (dot-notation nested set)
- `set field: interface.port value: 5678` on DEP-Z (dot-notation nested set)

**Assert:**
- Final FT-X front-matter: `phase: 3`, `adrs: [ADR-B, ADR-C]`, `domains-acknowledged: { security: "no trust boundary" }` (api removed)
- Final DEP-Z: `interface.port: 5678`, other interface fields preserved
- `remove` of a value not in the list is a no-op (no error, no change); verified by adding a parallel mutation `remove field: adrs value: ADR-NOT-PRESENT`
- Applying the same request twice yields identical file content