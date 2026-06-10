# Two Pillars conformance — clause mapping

How a Product repository satisfies "The Two Pillars — Specification for
Autonomous Engineering & Software Delivery" (working draft 0.1), clause by
clause. Product instantiates the **specification pillar**: features are
What units, ADRs are How units, TCs are the declared acceptance criteria,
and `product verify` is the declared judge. Run `product conformance check`
to evaluate the mechanically checkable clauses; this document declares the
disposition of every clause, so unchecked clauses are explicit, not hidden.

**Target profile: Level 3 — spec-driven.** A human reviews each unit of
output, so the Level 4/5 Execution Contract blocks are not required.

Dispositions:

- **checked** — evaluated by `product conformance check`; violations exit 1.
- **by construction** — guaranteed by the artifact model or graph loader;
  reported as passing in the conformance report.
- **process** — a mechanism exists in the toolchain but is not (yet) gated
  by the conformance command.
- **out of scope** — requires semantic judgment or Level 4/5 machinery; not
  evaluated at the Level 3 profile.

## §0 Preamble — discovery records

| Spec requirement | Disposition | Product mechanism |
|---|---|---|
| Exploratory findings enter engineered mode only via a frozen discovery record feeding a What specification | process | `product onboard` (ADR-027) discovers decisions from existing code into proposed ADRs; `product author` (ADR-022) runs graph-aware authoring sessions whose output is specification artifacts, not shipped code. A fused or informal document is migrated with `product migrate` — the split into features/ADRs is the promotion. |

## §4.0 Structural separation

| Clause | Disposition | Product mechanism |
|---|---|---|
| SPEC-SPLIT-1 | by construction (checked degenerate case) | Features (What) and ADRs (How) are distinct artifact kinds in distinct configured directories with distinct front-matter schemas. The conformance check flags the only collapsing configuration: `[paths].features` == `[paths].adrs`. |
| SPEC-SPLIT-2 | process | `product migrate` exists precisely to split fused documents into features + ADRs before they are used as input. Whether some upstream PRD remains fused is outside the graph's visibility. |

## §4.1 What specification (features)

| Clause | Disposition | Product mechanism |
|---|---|---|
| SPEC-WHAT-1 | **checked** | `name` in product config is the single system identity; W019 additionally warns when the responsibility statement suggests two products. |
| SPEC-WHAT-2 | **checked** (presence) | `[product].responsibility` (FT-039) declares purpose. Whether it answers problem/party/need "without inference" is semantic — out of scope. |
| SPEC-WHAT-3 / 3.1 | out of scope | The feature schema has no actor vocabulary. Actors live in feature prose; declaring and closing them needs a schema extension (candidate future work). |
| SPEC-WHAT-4 | **checked** (structural) | Every non-abandoned feature must carry a non-empty `Functional Specification` with `Behaviour` and `Error handling` subsections — behaviours with exception conditions. Full subsection vocabulary (Inputs, Outputs, State, Invariants, Boundaries) is enforced as W030 by `[features].required-sections` (FT-055, ADR-047). |
| SPEC-WHAT-5 | **checked** | Every non-abandoned feature must have a non-empty `## Out of scope` section. |
| SPEC-WHAT-6 | process | Data entities are declared in the `State` subsection (W030) and `docs/dependencies` DEP artifacts; there is no per-entity owner/lifetime schema. |
| SPEC-WHAT-7 | process | Constraints are carried as cross-cutting/platform ADRs with linked fitness TCs (G010 flags platform ADRs with no enforcement). |
| SPEC-WHAT-8 | **checked** | Every non-abandoned feature must link at least one TC (`tests:` or `validates.features`) — the testable acceptance criterion. |
| SPEC-WHAT-9 | out of scope | No open-questions schema on features. Open questions live in prose or the issue tracker. |

## §4.2 How specification (ADRs)

| Clause | Disposition | Product mechanism |
|---|---|---|
| SPEC-HOW-1 | by construction | The What reference is the bidirectional feature↔ADR edge; version reference is the sealed `content-hash` (ADR-032) plus the amendment audit trail. |
| SPEC-HOW-2 / 2.1 | **checked** (advisory) | One decision per ADR file. A top-level " and " in an accepted ADR title is flagged as a possible fused decision (same SRP heuristic as W019 and the code-quality `//!` gate). |
| SPEC-HOW-2.2 | by construction | The graph loader rejects dependency cycles (E003) and supersession cycles (E004) before any command runs. |
| SPEC-HOW-3 / 3.1 | out of scope | No declared data-model artifact kind to reconcile against What entities. |
| SPEC-HOW-4 | process | External integrations are DEP artifacts (ADR-030) with `interface` blocks; G008 flags a dependency used with no governing ADR. |
| SPEC-HOW-5 | **checked** | Every accepted ADR must document rejected alternatives (the stricter, MUST-severity sibling of gap rule G003). |
| SPEC-HOW-6 | process | The error-handling strategy is itself governed by accepted ADRs (ADR-013 error model) and the `error-handling` concern domain checked by `product preflight`. |
| SPEC-HOW-7 | process | Thresholds live in `[metrics.thresholds]` and TC-enforced fitness functions; no per-constraint NFR linkage. |
| SPEC-HOW-8 | out of scope | As SPEC-WHAT-9 — no open-questions schema. |

## §4.3 SPMC & derivation

| Clause | Disposition | Product mechanism |
|---|---|---|
| SPEC-SPMC-1 | process | `product context FT-XXX` assembles the Context element; Prompt templates are per-model data files (FT-063, ADR-049); Schema is the front-matter contract (`product schema`); Model is selected by `--target`. |
| SPEC-SPMC-2 | process | Bundles are BFS-bounded and assembled before execution; `--measure` records bundle dimensions to front-matter. Freezing-by-hash at execution time is future work. |
| SPEC-SPMC-3 | process | `product implement` runs one bundle per feature; the request log (FT-041, ADR-038) hashes every write request. Per-file-version single-writer attribution is Level 4 territory. |
| SPEC-DERIVE-1 | process | Impact analysis (`product impact`) and gap rule G006 surface weakly-anchored components; not a hard gate. |
| SPEC-DERIVE-2 | process | ADR-to-feature tracing is the `features:` edge; per-decision What-element anchors are prose. |
| SPEC-DERIVE-3 | **checked** | An accepted feature-specific ADR anchored to no feature is flagged as an undeclared product decision. Broader scopes (cross-cutting, platform, domain) anchor at the system level by declaration. |

## §5 Execution Contract (Level 4/5 — informative at Level 3)

At Level 3 a human inspects each unit of output, so §5 is not required for
conformance. Where Product already implements an execution-side mechanism,
it is listed for completeness:

| Clause | Disposition | Product mechanism |
|---|---|---|
| EXEC-VER-1/2 | process | `product verify` is the declared judge; TC verdicts (`passing` / `failing` / `unrunnable`) are declared artifacts in front-matter. |
| EXEC-VER-3 | process | Quality criteria per artifact kind: TCs for code, W030 sections for features, gap rules for ADRs — declared before judgment. |
| EXEC-VER-5 | process | The judge (`cargo-test` runners driven by `verify`) is not the generating agent. |
| EXEC-VER-7 | process | TC `validates:` edges trace every criterion to its specification elements. |
| EXEC-TRANS-1 | process | Verdict consequences: passing advances feature status, failing halts completion, E022 escalates missing runner config. |
| EXEC-CLOSE-4 | **checked** | A `complete` feature whose linked TC lacks a `passing` (or acknowledged `unrunnable`) verdict is output accepted without a verdict. |
| EXEC-WORK/CAP/ENV/TOOL/CRED/IN/OUT, EXEC-OUT-9, EXEC-CLOSE-6 | out of scope | Worker roles, capability grants, scoped credentials, provenance records, and outcome contracts are Level 4/5 machinery not present in the CLI. |

## Running the check

```bash
product conformance check                # text report, exit 1 on violations
product conformance check --format json  # CI-consumable report
```

The JSON report carries `spec` (`two-pillars/0.1`), `profile` (`level-3` or
`below-level-3`), per-clause outcomes, findings with suggested actions, and
summary counts. Advisories (disregarded SHOULDs) never affect the exit code.
