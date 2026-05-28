---
id: FT-104
title: Default-acknowledged cross-cutting ADRs with per-feature opt-out
phase: 6
status: complete
depends-on:
- FT-021
- FT-026
adrs:
- ADR-043
- ADR-048
- ADR-018
- ADR-042
- ADR-047
- ADR-051
tests:
- TC-860
- TC-861
- TC-862
- TC-863
domains:
- api
domains-acknowledged:
  api: "Adds one CLI verb (`feature reject`) and one new preflight JSON shape; no new MCP tool, no new domain concerns."
adrs-acknowledged:
  ADR-041: "FT-104 neither removes nor deprecates any prior surface; the new field is purely additive."
  ADR-049: "FT-104 does not touch the context bundle assembly path; preflight is a separate surface."
  ADR-050: "FT-104 does not introduce or modify any PAT artifact."
  ADR-040: "FT-104 changes the preflight signal but does not modify the unified verify pipeline structure."
---

## Description

Cross-cutting ADRs (`scope: cross-cutting`) apply to every
feature by default. Before FT-104, every feature had to either
explicitly list each one in its `adrs:` array or carry a
domain acknowledgement; missing the link surfaced as a preflight
gap. For near-universal concerns (error handling, logging,
config layout) this produced noise on every feature without
adding signal.

FT-104 splits the surface in two:

1. **Config-driven default acknowledgement.** A new
   `[features].default-acknowledged-cross-cutting` array in
   `product.toml` names cross-cutting ADRs that every feature
   acknowledges automatically. Preflight reports these with
   status `default-acknowledged` instead of `gap`.
2. **Explicit opt-out with rationale.** A feature that
   genuinely disagrees declares it via the new
   `adrs-rejected:` frontmatter (list of `{id, reason}`),
   written through `product feature reject <ADR> --feature
   <FT> --reason "..."`. Rejections re-introduce the gap with
   a distinct status (`intentional`) so reviewers see "we
   thought about this and said no" rather than "we forgot."

A graph-check pass (`ft104_drift`) surfaces three drift forms
as warnings (W036/W037/W038) so the catalog stays honest when
the underlying ADRs move.

---

## Functional Specification

### Inputs

- `[features].default-acknowledged-cross-cutting: Vec<String>`
  in `product.toml` (default empty).
- `adrs-rejected: Vec<{id, reason}>` in feature frontmatter
  (default empty).
- The live ADR catalog.

### Outputs

- `product preflight FT-XXX` (text + `--format json`) renders
  the four statuses: `linked`, `acknowledged` (existing),
  `default-acknowledged` (new), `intentional` (new), `gap`.
- `product feature reject <ADR> --feature <FT> --reason "..."`
  writes the rejection to disk atomically.
- `product graph check` emits W036/W037/W038 on drift; exit
  code stays at the warnings level (never error).

### State

- Feature files gain an optional `adrs-rejected:` block when
  the operator rejects an ADR.
- `product.toml` gains a single optional list under
  `[features]`. No other on-disk state.

### Behaviour

1. **Default acknowledgement is config-driven, not frontmatter-
   driven.** The feature file is never modified by adding an
   ADR to the default-ack list. A consumer reading the feature
   alone cannot tell which cross-cutting ADRs cover it — they
   must consult `product.toml` (this is intentional; one row
   in config beats N rows of feature noise).
2. **Default-acknowledged closes the gap.** Preflight returns
   exit 0 for a feature whose only missing ADRs are all in
   the default-ack list; the row renders `default-acknowledged`.
3. **Rejection re-opens the gap as `intentional`.** A feature
   that lists ADR-X in `adrs-rejected:` has its preflight row
   render `INTENTIONAL: <reason>` in text and `status:
   "intentional"` with the reason in JSON; the exit code is
   1 (the gap is real, just deliberate). A non-clean preflight
   blocks `product implement`.
4. **Empty reasons are rejected.** `product feature reject` and
   the underlying frontmatter parse refuse an empty reason
   (E011) — the rationale is load-bearing for the operator
   reading the JSON output later.
5. **Idempotent CLI.** Re-running `feature reject` with a new
   reason overwrites the existing entry rather than appending
   a duplicate.
6. **Drift detection.** `product graph check` invokes
   `ft104_drift::check_default_ack_drift`:
   - **W036** — the default-ack list names an ADR that no
     longer exists in `docs/adrs/`.
   - **W037** — the default-ack list names an ADR whose scope
     has changed away from `cross-cutting`.
   - **W038** — a feature's `adrs-rejected:` names an ADR not
     in the default-ack list (the rejection is a no-op).

### Invariants

- The default-ack list never mutates feature frontmatter.
- `adrs-rejected:` entries always carry a non-empty reason.
- Drift findings are warnings, never errors.
- The four new preflight statuses are mutually exclusive per
  (feature, ADR) pair: `linked > acknowledged >
  default-acknowledged > intentional > gap` (highest match
  wins; rejection overrides default-ack).

### Error handling

- **E011** — empty rejection reason (CLI + parser).
- The four new `CoverageStatus` variants render in both text
  (`render_preflight`) and JSON (`product preflight --format
  json`) paths without panicking.

### Boundaries

- **In scope:** config-driven default acknowledgement, the
  `feature reject` verb, the JSON preflight format, drift
  detection (W036/W037/W038).
- **Out of scope:** propagating default-ack to domain ADRs
  (FT-104 is cross-cutting only). Domain ADRs continue to
  require explicit `domains-acknowledged:` entries.
- **Out of scope:** retroactive cleanup of existing features'
  `adrs:` arrays once their entries become default-acknowledged
  (operators do this manually).
- **Out of scope:** MCP tool surface for `feature reject` — the
  CLI is sufficient for v1.

---

## Out of scope

- A `product feature unreject` verb (operators edit the file
  or remove the entry via the generic `request_apply`).
- A `default-acknowledged-domain` analogue. Domain ADRs are
  expected to be feature-specific; bulk acknowledgement of a
  domain across the whole repo is a smell, not an ergonomic
  win.

---

## Implementation notes

- The new field lives on `FeatureFrontMatter` and serialises
  as kebab-case `adrs-rejected` to match the existing schema
  convention.
- `domains::preflight` gained a 4th argument
  (`default_acknowledged: &[String]`); all callers
  (`commands/preflight`, `commands/author`,
  `author/preflight_gate`, `implement/pipeline`, MCP
  `health_handlers/preflight`) pass
  `config.features.default_acknowledged_cross_cutting`.
- The `commands/preflight` handler was split into
  `print_preflight_json`, `render_dep_availability`, and
  `probe_dep` helpers to stay under the 40-statement function
  budget (`tests/code_quality_tests.rs`).
- `src/domains/ft104_drift.rs` is a single function consumed
  by `graph::full_check::run`.

---

## Acceptance criteria

1. `product preflight FT-XXX` returns exit 0 when every
   missing cross-cutting ADR is in the default-ack list.
2. `product feature reject <ADR> --feature <FT> --reason
   "..."` writes the rejection atomically and rejects empty
   reasons.
3. Preflight renders `default-acknowledged` and `intentional`
   in both text and JSON outputs.
4. `product graph check` emits W036/W037/W038 on the three
   drift forms; exit code is not `1` (errors).
5. TC-860, TC-861, TC-862 all pass under `cargo t`.
6. The full suite (`cargo t`) is green; lib/bin clippy clean.
