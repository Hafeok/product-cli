---
id: TC-600
title: removal_deprecation_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_600_removal_deprecation_exit
---

## Exit Criteria — FT-047 Removal & Deprecation Tracking

FT-047 is complete when all of the following hold:

1. `tc-type: absence` is accepted by the parser, the request validator, and
   the schema render (TC-586 through TC-588).
2. ADR front-matter `removes:` and `deprecates:` fields parse, round-trip,
   and default to `[]` when absent (TC-589, TC-590).
3. `product gap check` emits G009 (severity high) for any ADR with non-empty
   `removes:` or `deprecates:` and no linked absence TC (TC-591).
4. `product graph check` emits W022 for the same condition (TC-592).
5. Linking an absence TC clears both G009 and W022 (TC-593).
6. The front-matter parser emits W023 for any field whose name appears in
   the `deprecates:` list of any accepted ADR (TC-594).
7. W023 never blocks: the field is still parsed, the graph still builds,
   exit code is 2 (warning) at most (TC-595).
8. W023 messages name the deprecating ADR by ID (TC-596).
9. The migration lifecycle works end-to-end: phase-1 deprecation TC passes
   (TC-597), phase-2 absence TC passes (TC-598), phase-2 with phase-1
   marked `unrunnable` does not block (TC-599).
10. Absence TCs run via `product verify --platform` and only via that
    pipeline; they are not collected by per-feature `product verify FT-XXX`
    (TC-588).
11. The schema documentation (`product agent-context`, `product_schema`)
    renders the new TC type and the two new ADR fields.
12. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and
    `cargo build` all pass.
13. Every TC under FT-047 has `runner: cargo-test` and `runner-args` set to
    the integration test function name.
