---
id: TC-726
title: nested subcommand groups listed alphabetically
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_726_nested_subcommand_groups_listed_alphabetically
---

## Scenario

Given a working `product` binary, when a user runs the `--help` flag on
each grouped subcommand (`product feature --help`, `product adr --help`,
etc.), then each rendered subcommand list must appear in ASCII-sorted
order.

## Procedure

For each group in: `feature`, `adr`, `test`, `dep`, `graph`,
`checklist`, `migrate`, `gap`, `author`, `prompts`, `drift`, `tags`,
`metrics`, `onboard`, `hash`, `request`:

1. Run `product <group> --help`.
2. Parse the `Commands:` section.
3. Filter out clap's auto-injected `help` row (or include it; either
   choice is consistent — pick one and apply it uniformly).
4. Assert the remaining sequence equals its own sort.

A failure in any group fails the whole test, naming the group and the
first out-of-order pair so the developer fixes it directly.

## Expected

Every group renders its nested subcommands in alphabetical order. No
group is skipped; the test enumerates them explicitly so adding a new
group requires updating the test (which is fine — the test is the
contract).

## Implementation hint

Implement as `tc_726_nested_subcommand_groups_listed_alphabetically`
in `tests/integration.rs`. Use a slice of group names and a single
helper that runs `--help` and asserts sortedness, called once per
group inside a loop.
