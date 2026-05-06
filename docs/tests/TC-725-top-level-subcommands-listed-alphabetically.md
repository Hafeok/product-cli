---
id: TC-725
title: top level subcommands listed alphabetically
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_725_top_level_subcommands_listed_alphabetically
---

## Scenario

Given a working `product` binary built from the repository, when a user
runs `product --help`, then the rendered subcommand list must appear in
ASCII-sorted order (case-sensitive, kebab-case names).

## Procedure

1. `cargo build --bin product`
2. Capture stdout of `product --help`.
3. Parse the `Commands:` section — every line of the form `  <name>`
   under that heading is a subcommand.
4. Extract just the subcommand names (the first whitespace-separated
   token on each line).
5. Assert the sequence equals its own sort under `str::cmp`.

## Expected

The asserted sequence equals (alphabetical, ASCII):

```
adr, agent-init, author, checklist, completions, context, cycle-times,
dep, drift, feature, forecast, gap, graph, hash, help, impact,
implement, init, install-hooks, mcp, metrics, migrate, onboard,
preflight, prompts, request, schema, status, tags, test, verify
```

(`help` appears because clap injects it; the test must allow for it
and either include it in the sort assertion or filter it out before
sorting.)

## Implementation hint

Implement as `tc_725_top_level_subcommands_listed_alphabetically` in
`tests/integration.rs` using `assert_cmd::Command::cargo_bin("product")`
with arg `--help`.
