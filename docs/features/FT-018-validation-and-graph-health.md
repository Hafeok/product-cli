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
domains: [data-model, error-handling]
domains-acknowledged:
  error-handling: >
    Validation diagnostics (E0xx/W0xx) use the error model from ADR-013
    which is linked via FT-010. The diagnostic format and exit codes are
    already governed; no separate error-handling ADR is needed here.
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