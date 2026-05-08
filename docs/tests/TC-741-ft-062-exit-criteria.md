---
id: TC-741
title: FT-062 exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-038
  - ADR-037
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_741_ft_062_exit_criteria
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Exit criteria — FT-062 MCP Parity for `depends-on` and Strict Request Shape Validation

FT-062 is complete when all of the following hold:

1. `product feature depends-on FT-XXX --add FT-YYY` writes the link
   atomically with cycle detection and broken-link validation
   (TC-733/734/735/737).
2. The MCP `product_feature_depends_on` tool exists, is registered in
   the registry, and behaves identically to the CLI (TC-733).
3. The existing `product_feature_link` MCP tool accepts an optional
   `feature` argument and delegates to the same plan helper (TC-736).
4. `product request validate` and `product request apply` reject
   unknown top-level keys with **E025** (TC-738).
5. `product request validate` and `product request apply` reject
   unknown mutation fields with **E026** (TC-739).
6. Dot-notation mutation paths whose head segment is a known field are
   still accepted (TC-740).
7. `product_schema` returns a `fields` map sourced from the same
   constants the request validator consults — `field_schema` is the
   single source of truth (TC-732).
8. AGENTS.md "Key MCP Tools" table lists `product_feature_depends_on`.
9. `cargo t`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and
   `cargo build` all pass.
10. `product graph check` exits `0` on the live repository after the
    feature lands.