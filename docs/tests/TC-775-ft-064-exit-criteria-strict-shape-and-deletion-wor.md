---
id: TC-775
title: FT-064 exit criteria — strict shape and deletion work end-to-end
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-064
  adrs: []
phase: 5
---

Consolidated exit-criteria check for FT-064. The feature is done
when:

1. Every mis-shaped change (op/field/value at change level) is
   rejected before any file is touched (TC-770).
2. Every empty-mutations change is rejected (TC-771).
3. Every unknown key inside a mutation is rejected (TC-772).
4. `op: remove` on a list-valued feature field actually removes the
   entry and reports `mutations >= 1` (TC-773).
5. An artifact can be deleted through the request interface or a
   dedicated CLI/MCP surface, the file is unlinked, and the
   deletion lands in `requests.jsonl` with a valid hash-chain
   link (TC-774).
6. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` all pass.
7. `product graph check` exits 0 against the post-feature repo.
