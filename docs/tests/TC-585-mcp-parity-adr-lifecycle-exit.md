---
id: TC-585
title: mcp_parity_adr_lifecycle_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
  - ADR-032
phase: 1
runner: cargo-test
runner-args: tc_585_mcp_parity_adr_lifecycle_exit
---

## Exit Criteria — FT-046 MCP Parity for ADR Lifecycle Operations

FT-046 is complete when all of the following hold:

1. `product_adr_amend` accepts an optional `body` parameter. When present, the on-disk body, `content-hash`, and `amendments` array are updated atomically in one MCP call (TC-577).
2. `product_adr_amend` returns `E017 amendment-nothing-changed` when the supplied body is byte-identical to the on-disk body (TC-579).
3. `product_adr_amend` returns `E019 amendment-carries-status` when the payload also attempts to change `status` — file is unchanged (TC-578).
4. `product_adr_amend` returns `E018 amendment-not-accepted` when called on an ADR whose status is not `accepted` — file is unchanged (implicit; covered by existing amend code path).
5. `product_adr_status` writes every non-`accepted` transition to disk: `proposed → abandoned`, `proposed → superseded`, `accepted → abandoned`, `accepted → superseded` (TC-580, TC-582, TC-583).
6. `product_adr_status` with `status: accepted` returns `E020 status-accepted-is-manual` and names the exact CLI command to run — file is unchanged (TC-581).
7. `product_adr_status` refuses `accepted → proposed` with `E021 status-cannot-demote-accepted` — file is unchanged (TC-584).
8. Supersession via MCP writes the bidirectional link (target's `supersedes` is updated in the same atomic batch per ADR-015). Cycle detection rejects with `E004` and both files are preserved (TC-582).
9. The success response shape is consistent across all lifecycle tools: `{ id, status, content-hash?, amendments?, superseded-by? }`. No more `{ note: "Use CLI..." }` divergence.
10. `product graph check` exits `0` after every successful transition.
11. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
12. Every TC in this feature (TC-577 through TC-585) has `runner: cargo-test` and `runner-args: tc_XXX_snake_case` matching the Rust test function name.
