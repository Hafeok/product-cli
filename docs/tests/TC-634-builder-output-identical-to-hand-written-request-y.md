---
id: TC-634
title: builder_output_identical_to_hand_written_request_yaml
type: scenario
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Session — builder-yaml-equivalence

### Given

Two equivalent intents: (a) a hand-written request YAML file
describing one feature + one ADR + one TC with all cross-links,
and (b) the same intent expressed via the builder's `add`
subcommands in arbitrary order.

### When

Both YAMLs are passed to `product request validate` and then to
`product request apply` against fresh-clone fixtures.

### Then

- `product request validate` produces the same findings set for
  both (order-independent comparison on E-class and W-class
  codes and their `location:` paths after
  `ref:` name normalisation).
- `product request apply` writes the same set of artifact files
  with the same cross-references in both cases.
- The resulting graphs (parsed front-matter) are structurally
  identical modulo ID assignment order.
