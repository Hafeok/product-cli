---
id: TC-616
title: tc_types_system_exit
type: exit-criteria
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_616_tc_types_system_exit"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Exit Criteria — FT-048 TC Type System

FT-048 is complete when all of the following hold:

1. The four structural types (`exit-criteria`, `invariant`, `chaos`,
   `absence`) drive their respective mechanics by exact-string match
   (TC-601, TC-602, TC-603, TC-604).
2. `[tc-types].custom` in `product.toml` accepts a list of custom type
   names; configured custom types pass type validation (TC-605).
3. Unknown TC types (neither built-in nor configured) emit E006 with a hint
   listing both sets and a `product request change` snippet (TC-606,
   TC-615).
4. Custom types behave identically to `scenario` in all Product mechanics:
   bundle inclusion, runner execution, status tracking; they trigger no
   W004 / G002 / G009 (TC-607).
5. Custom types appear in the `product agent-init` / `agent-context` schema
   render with the `(custom — this project)` annotation (TC-608).
6. The context bundle TC ordering is
   `exit-criteria → invariant → chaos → absence → scenario → benchmark →
   [custom alphabetical]` (TC-609, TC-612, TC-613).
7. A reserved structural name in `[tc-types].custom` triggers E017 at
   startup before any subcommand runs (TC-610, TC-611).
8. `product request validate/apply` enforces the same type validation as
   `product graph check` (TC-614, TC-615).
9. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and
   `cargo build` all pass.
10. Every TC under FT-048 has `runner: cargo-test` and `runner-args` set to
    the integration test function name.