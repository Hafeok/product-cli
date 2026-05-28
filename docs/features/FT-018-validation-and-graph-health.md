---
id: FT-018
title: Validation and Graph Health
phase: 1
status: complete
depends-on: []
adrs:
- ADR-010
- ADR-025
tests:
- TC-031
- TC-032
- TC-033
- TC-034
- TC-132
- TC-133
- TC-134
- TC-135
- TC-136
- TC-137
- TC-138
- TC-139
- TC-715
domains:
- data-model
- error-handling
domains-acknowledged:
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  error-handling: Validation diagnostics (E0xx/W0xx) use the error model from ADR-013 which is linked via FT-010. The diagnostic format and exit codes are already governed; no separate error-handling ADR is needed here.
---

`product graph check` is the primary consistency tool. All output goes to stderr. Exit codes follow the three-tier scheme from ADR-009 and ADR-013.

Errors (exit code 1):

| Code | Condition |
|---|---|
| E002 | Broken link — referenced artifact does not exist |
| E003 | Dependency cycle in `depends-on` DAG |
| E004 | Supersession cycle in ADR `supersedes` chain |
| E001 | Malformed front-matter in any artifact file |
| E011 | `domains-acknowledged` entry present with empty reasoning |
| E012 | Domain declared in front-matter not present in `product.toml` vocabulary |

Warnings (exit code 2 when no errors):

| Code | Condition |
|---|---|
| W001 | Orphaned artifact — ADR or test with no incoming feature links |
| W002 | Feature has no linked test criteria |
| W003 | Feature has no test of type `exit-criteria` |
| W004 | Invariant or chaos test missing formal specification blocks |
| W005 | Phase label disagrees with topological dependency order |
| W006 | Evidence block `δ` below 0.7 (low-confidence specification) |
| W007 | Schema upgrade available |
| W008 | Migration: ADR status field not found, defaulted to `proposed` |
| W009 | Migration: no test subsection found in ADR, no TC files extracted |
| W010 | Cross-cutting ADR not linked or acknowledged by a feature |
| W011 | Feature declares a domain with existing domain-scoped ADRs but no coverage |

Schema errors (exit code 1):

| Code | Condition |
|---|---|
| E008 | `schema-version` in `product.toml` exceeds this binary's supported version |

Gap analysis codes (stdout, separate from `graph check`):

| Code | Severity | Condition |
|---|---|---|
| G001 | high | Testable claim in ADR body with no linked TC |
| G002 | high | Formal invariant block with no scenario or chaos TC |
| G003 | medium | ADR has no rejected alternatives section |
| G004 | medium | Rationale references undocumented external constraint |
| G005 | high | Logical contradiction between this ADR and a linked ADR |
| G006 | medium | Feature aspect not addressed by any linked ADR |
| G007 | low | Rationale references decisions superseded by a newer ADR |

All errors use the rustc-style diagnostic format (file path, line number, offending content, remediation hint). `--format json` outputs structured JSON to stderr for CI consumption. See ADR-013 for the full error model.

---

---

## Description

`product graph check` is the primary consistency tool. It runs all structural validations over the knowledge graph, emitting rustc-style diagnostics (file path, line number, offending content, remediation hint) to stderr. Exit codes follow the three-tier scheme from ADR-009: 0 (clean), 1 (errors), 2 (warnings only). The feature also covers auto-orphaning of test criteria when a feature is abandoned (ADR-010) and the gap analysis codes surfaced by `product gap check` (stdout, separate from graph check).

## Functional Specification

### Inputs

- The complete in-memory knowledge graph built from all artifact files
- `--format json`: optional flag to emit structured JSON diagnostics to stderr for CI consumption
- Feature status transitions via `product feature status FT-XXX abandoned`: triggers the auto-orphan cascade

### Outputs

- `product graph check` — all diagnostics to stderr; exit code 0, 1, or 2
- `product gap check` — gap analysis codes (G001–G007) to stdout; separate command, does not affect graph-check exit code
- Auto-orphan side effect: when a feature is set to `abandoned`, all linked TC `validates.features` lists are mutated atomically to remove that feature's ID; the developer sees a stdout summary of which TCs were modified

### State

Stateless. `product graph check` is a read-only command — it does not modify any files. The auto-orphan cascade is a write side effect of `product feature status ... abandoned`, not of `graph check` itself.

### Behaviour

1. Build the in-memory graph.
2. Run all validation checks in sequence: parse errors, duplicate IDs, broken links (E002), dependency cycles (E003), supersession cycles (E004), malformed front-matter (E001), acknowledgement without reasoning (E011), unknown domain vocabulary (E012), orphaned ADRs (W001), features with no tests (W002), features with no exit-criteria TC (W003), formal blocks missing for invariant/chaos TCs (W004), phase/dependency order disagreement (W005), evidence `δ` below 0.7 (W006), schema upgrade available (W007), cross-cutting ADR unacknowledged (W010), domain gap without acknowledgement (W011), TC runner missing (blocks verify, reported as E022 by verify).
3. Emit each finding as a rustc-style diagnostic to stderr.
4. Exit: 0 if no findings, 1 if any errors, 2 if warnings only (ADR-009).
5. Auto-orphan (on `product feature status FT-XXX abandoned`): remove the feature ID from `validates.features` in all linked TC files atomically; log each modified TC to stdout; TCs with empty `validates.features` thereafter become W001 orphans on next `graph check`.
6. After verify: failing TCs must remain visible as full graph nodes — a failing `product verify` must never corrupt TC front-matter to the point where TCs become invisible to `graph check` (TC-715).

### Invariants

- All diagnostic output goes to stderr; stdout is clean (allowing `product graph check 2>/dev/null` to suppress diagnostics while preserving exit code semantics).
- A `product verify` run that records a failing TC must not cause any new E001 parse errors on subsequent `graph check` runs (TC-715).
- After abandoning a feature, `product graph check` exits with code 2 (warnings, orphaned TCs), never code 1 (broken links) — because auto-orphaning cleans the edges (TC-032).
- Gap analysis (G-codes) is emitted to stdout by `product gap check`, not mixed into the stderr diagnostic stream of `product graph check`.

### Error handling

All errors follow the rustc-style diagnostic format defined in ADR-013:
```
error[E002]: broken link
  --> docs/features/FT-009.md:12
   |
12 | adrs: [ADR-099]
   |        ^^^^^^^ referenced ADR-099 does not exist
   = hint: run `product adr new ADR-099 --title "..."` to create it
```
`--format json` replaces the human-readable format with a structured JSON array on stderr.

### Boundaries

- `product graph check` is read-only; it never modifies artifact files.
- Gap analysis (`G001`–`G007`) is a separate command (`product gap check`) with separate output stream and exit codes.
- Domain coverage checks (`W010`, `W011`) surface gaps; they do not auto-fix or block commands other than `product implement` (where preflight is a gate).

## Out of scope

- Automatically resolving broken links or missing TCs (the tool reports, does not repair).
- Linting prose content of ADR bodies or feature descriptions (only structural front-matter and graph topology are validated).
- Schema migration (that is `product migrate schema`, FT-020).
