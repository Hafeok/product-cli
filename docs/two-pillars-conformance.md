# Two Pillars conformance — the checked clause set

`product conformance check` evaluates the knowledge graph against the checkable
subset of the **Two Pillars Specification Framework** (Level 3). This page is
the clause map: what each clause means and how it's evaluated. The check is
implemented in `product-core/src/conformance/` (FT-108, ADR-052).

## How the check runs

- **Exit 0** — every MUST clause holds.
- **Exit 1** — at least one MUST clause is violated.
- Advisories (disregarded SHOULDs) are reported but never fail a clause.

Two evaluation modes:

| Mode | Meaning |
|---|---|
| **By construction** | The clause holds the moment the graph loads (the parser/graph model makes the violation unrepresentable). Reported as passing once the graph is well-formed. |
| **Checked** | Evaluated against the loaded graph; a finding for the clause fails it. |

## The clauses

| Clause | Title | Mode | What it confirms |
|---|---|---|---|
| **SPEC-SPLIT-1** | What plus How are separate artifacts | By construction | What artifacts (features) live apart from How artifacts (ADRs/patterns) — the pillars are never fused. |
| **SPEC-WHAT-1** | Single declared system identity | Checked | The product declares exactly one identity (`name` in config). |
| **SPEC-WHAT-2** | Declared purpose for the system | Checked | A responsibility/purpose statement is present (FT-039). |
| **SPEC-WHAT-4** | Behaviours declare error handling | Checked | Each behaviour spec states how it fails, not only the happy path. |
| **SPEC-WHAT-5** | Non-empty out-of-scope declaration | Checked | Each feature declares what it is *not* — scope is bounded explicitly. |
| **SPEC-WHAT-8** | Acceptance criterion per behaviour | Checked | Every behaviour carries at least one acceptance criterion (a TC). |
| **SPEC-HOW-2.1** | One responsibility per decision | Checked | Each ADR addresses a single decision (the SRP for decisions). |
| **SPEC-HOW-2.2** | Acyclic dependency graph | By construction | The decision/dependency graph has no cycles. |
| **SPEC-HOW-5** | Decisions record rejected alternatives | Checked | Each ADR documents the alternatives it rejected. |
| **SPEC-DERIVE-3** | No undeclared product decisions | Checked | Significant decisions are captured as ADRs, not left implicit in code. |
| **EXEC-CLOSE-4** | Output judged before acceptance | Checked | No artifact is accepted without a verdict — verification gates closure. |

## Relationship to the framework graph

The clauses above govern the **meta graph** (features/ADRs/TCs) — the discipline
of *how this repository specifies itself*. The deeper **framework graph**
(What/How/Delivery — bounded contexts, deciders, slices) carries its own,
richer conformance via the SHACL-shaped rules in `product-core/src/pf/` and the
`product domain validate` / `product decider validate` / seam checks. See
[product-framework-open.md](product-framework-open.md) for the normative model and
[guide/framework-concepts.md](guide/framework-concepts.md) for the primer.

## Running it

```bash
product conformance check            # text report, exit 1 on any MUST violation
product conformance check --format json
```
