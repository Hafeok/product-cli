---
id: TC-622
title: mcp_body_update_dep_exit
type: exit-criteria
status: passing
validates:
  features:
  - FT-050
  adrs:
  - ADR-030
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_622_mcp_body_update_dep_exit"
last-run: 2026-04-20T13:27:05.074524328+00:00
last-run-duration: 0.2s
---

## Exit Criteria — FT-050 MCP body_update Supports Dependencies

FT-050 is complete when all of the following hold:

1. `product_body_update` accepts `DEP-NNN` IDs and replaces the dep body
   atomically via `fileops::write_file_atomic`, preserving front-matter
   byte-for-byte (TC-620).
2. Unknown dep IDs produce an error message naming the missing dep, in
   parity with feature / ADR / TC wording (TC-621).
3. Unknown prefixes still hit the existing fallback error — no silent
   behaviour change (TC-621).
4. The tool description in `src/mcp/tools.rs` for `product_body_update`
   lists `DEP-NNN` in its `id` hint so the capability is discoverable.
5. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and
   `cargo build` all pass.
6. Every TC under FT-050 has `runner: cargo-test` and `runner-args` set to
   the integration test function name.