---
id: FT-125
title: product status surfaces the framework What, How, and delivery graph
phase: 6
status: complete
depends-on:
- FT-124
adrs:
- ADR-066
tests:
- TC-966
domains:
- api
domains-acknowledged:
  ADR-041: Additive — `product status` gains a section; the legacy FT/ADR/TC summary is unchanged, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: A display-only read; the gathering + rendering live in the status adapter, no new pure slice is warranted.
  ADR-048: Reads the captured What graph + How contract + delivery dirs; writes nothing.
  ADR-051: The TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: One scenario TC drives the binary through assert_cmd. No property or session dimension for a display section.
  ADR-040: A read-only summary over the existing framework artifacts; it touches no verification gate.
patterns:
- PAT-001
---

## Description

`product status` summarised only the legacy FT/ADR/TC graph (the product-cli
tool's own spec). With the framework graph now carrying the real product —
What (domain + event model + deciders), How (decisions/principles/patterns/
contracts/layout), and delivery (slices) — `status` gains a **Framework graph**
section so the new graph is visible at a glance alongside the legacy summary.

## Functional Specification

### Behaviour

When framework artifacts exist for the default product, `product status` appends:

```
── Framework graph ──
What: <n> contexts, <n> entities, <n> events, <n> commands, <n> deciders
How: <n> decisions, <n> principles, <n> patterns, <n> contracts, <n> layout rules
Delivery: <n> slices
```

- **What** counts come from the captured domain session; **deciders** from
  `.product/deciders/`.
- **How** counts come from `.product/how-contract.yaml` (decisions, principles,
  patterns, contracts = application + optional infrastructure) and layout-rule
  count from `.product/layout.yaml`.
- **Delivery** counts the slices in `.product/slices/`.
- `--format json` adds a `framework` object mirroring the section.

### Error handling

- If no framework artifacts exist, the section is omitted entirely — legacy
  repos see no change; its absence is not an error.
- A missing What graph renders `What: (none captured)`; a missing How renders
  `How: (none)`. Malformed files are skipped with their counts treated as zero,
  so `status` always renders the legacy summary.

## Out of scope

- Per-feature/per-release rollups and the `done` predicate (§7.2) — separate
  delivery increments.

## Acceptance

- TC-966 — with a captured What graph and a slice, `product status` shows the
  Framework graph section with the correct counts.
