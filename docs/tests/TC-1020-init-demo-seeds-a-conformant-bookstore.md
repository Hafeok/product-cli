---
id: TC-1020
title: init demo seeds a conformant bookstore
type: scenario
status: passing
validates:
  features:
  - FT-144
  adrs:
  - ADR-088
  - ADR-048
phase: 8
observes:
- stdout
- exit-code
- disk-state
runner: cargo-test
runner-args: tc_1020_init_demo_seeds_a_conformant_bookstore
last-run: 2026-06-19T15:22:08.028853574+00:00
last-run-duration: 0.4s
---

## Scenario — `init --demo` writes a real, conformant What model

**Given** an empty directory,
**When** the user runs `product init --yes --name bookstore --demo`,
**Then** the process exits 0 and stdout reports `Seeded the bookstore demo`.

**And when** the user runs `product domain validate`, **then** it exits 0 and
reports `conformant` — the seeded graph on disk is a real, validated What model.

**And** `product guide` shows `[x] Captured a What model` and
`[x] What is conformant`.

## Validates

- FT-144 — framework-aware init — signposting and a --demo bookstore seed
- ADR-088 — framework-graph onboarding is a derived guide plus a signposted, seedable init
- ADR-048 — canonical `.product/` repository layout