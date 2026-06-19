---
id: TC-945
title: archetype check layout against the tree
type: scenario
status: passing
validates:
  features:
  - FT-120
  adrs:
  - ADR-086
phase: 6
observes:
- exit-code
- stdout
- stderr
runner: cargo-test
runner-args: tc_945_archetype_check_layout_against_the_tree
---

## Scenario — apply an archetype's layout model to the repository tree

**Given** an archetype `chk` whose `layout.yaml` requires `product.toml`
(`must_exist`, exactly 1) and forbids `**/*.secrets.*` (`must_not_exist`),
**When** the user runs `product archetype check chk` against a tree that has
`product.toml` and no secrets,
**Then** the process exits 0 and stdout reports `layout-conformant`.

**And when** a `config.secrets.json` file is added to the tree and the check is
re-run, **then** the process exits 1 and stderr reports the `must_not_exist`
violation.

## Validates

- FT-120 — product archetype check — enforce a layout model against the repository tree
- ADR-086 — Layout conformance applies glob rules to the filesystem with allowlist semantics
