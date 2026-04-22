---
id: FT-029
title: Gap Analysis
phase: 4
status: complete
depends-on: []
adrs:
- ADR-006
- ADR-019
tests:
- TC-086
- TC-087
- TC-088
- TC-089
- TC-090
- TC-091
- TC-092
- TC-093
- TC-094
- TC-095
- TC-096
- TC-097
- TC-098
domains:
- api
- observability
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

Gap analysis is the continuous LLM-driven process of identifying specification incompleteness, inconsistency, and missing coverage in the repository's ADRs. It runs in CI against changed ADRs and produces structured findings that are tracked over time.

### What Gap Analysis Checks

Gap analysis checks seven classes of gap, each with a code and a severity:

| Code | Severity | Description |
|---|---|---|
| G001 | high | **Missing test coverage** — ADR makes a testable claim with no linked TC |
| G002 | high | **Untested formal invariant** — `⟦Γ:Invariants⟧` block exists but no scenario or chaos TC exercises it |
| G003 | medium | **Missing rejected alternatives** — ADR has no documented rejected alternatives |
| G004 | medium | **Undocumented constraint** — ADR rationale references an external constraint not captured in any linked artifact |
| G005 | high | **Architectural contradiction** — this ADR makes a claim logically inconsistent with a linked ADR |
| G006 | medium | **Feature coverage gap** — a feature aspect is not addressed by any linked ADR |
| G007 | low | **Stale rationale** — ADR rationale references something contradicted by a more recent superseding ADR |

### Context Used for Analysis

Each ADR is analysed with its full depth-2 context bundle — the ADR, all linked features, all linked test criteria, and all related ADRs reachable within 2 hops. This is the same bundle an implementation agent would receive, which means gap analysis validates the context bundle's completeness from the same perspective.

### Output Format

Gap findings are structured JSON, written to stdout (not stderr — they are results, not errors):

```json
{
  "adr": "ADR-002",
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "findings": [
    {
      "id": "GAP-ADR002-001",
      "code": "G001",
      "severity": "high",
      "description": "The invariant 'exactly one leader at all times' stated in the rationale has no linked chaos test exercising a split-brain scenario.",
      "affected_artifacts": ["ADR-002"],
      "suggested_action": "Add a chaos TC validating leader uniqueness under network partition.",
      "suppressed": false
    }
  ],
  "summary": { "high": 1, "medium": 0, "low": 0, "suppressed": 0 }
}
```

### The Baseline File

`gaps.json` at the repository root tracks gap state across runs:

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "GAP-ADR002-001",
      "reason": "Split-brain chaos test deferred to phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ],
  "resolved": [
    {
      "id": "GAP-ADR001-003",
      "resolved_at": "2026-04-12T14:30:00Z",
      "resolving_commit": "git:def456"
    }
  ]
}
```

A gap is **new** if its ID does not appear in `gaps.json`. A gap is **suppressed** if it appears in `suppressions`. A gap is **resolved** if it was previously suppressed or known and is no longer detected. Only new unsuppressed gaps cause CI to exit 1.

### Gap IDs

Gap IDs are deterministic and stable. They are derived from: the ADR ID, the gap code, and a hash of the affected artifact IDs and gap description. The same logical gap detected on two different runs produces the same ID. This is critical for suppression to work correctly — a suppressed gap must remain suppressed across runs.

```
GAP-{ADR_ID}-{GAP_CODE}-{SHORT_HASH}
e.g. GAP-ADR002-G001-a3f9
```

### CI Integration

The `--changed` flag is the primary CI mode. It uses `git diff --name-only HEAD~1` to identify changed ADR files, then expands to include 1-hop graph neighbours (ADRs that share a feature with any changed ADR). This scoping strategy ensures that:

- A PR that modifies ADR-002 also analyses ADR-005 if they share a feature (because the change may create new inconsistencies between them)
- The analysis set is bounded — at most `changed_adrs × avg_neighbour_count` ADRs are analysed per run
- Unrelated ADRs are never analysed, keeping CI cost proportional to change scope

### LLM Prompt Design

The gap analysis prompt is fixed and versioned. It does not change between runs unless a new version is explicitly released. This ensures that the same repository state produces comparable findings across runs.

The prompt instructs the model to:
1. Read the context bundle
2. Check only for the seven defined gap types — not for general quality issues
3. Respond only in the specified JSON schema — no prose preamble
4. Assign the deterministic gap ID format
5. For G005 (contradiction), cite the specific claims from both ADRs that conflict

The prompt is stored at `benchmarks/prompts/gap-analysis-v1.md` and referenced by version in `product.toml`:

```toml
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"   # gaps below this severity are informational only
```

### Determinism Strategy

LLM output is non-deterministic. Three measures stabilise gap analysis for CI:

1. **Temperature=0** for all gap analysis calls
2. **Structured JSON output only** — the model is instructed to produce only JSON matching the schema. Findings that cannot be parsed into the schema are discarded with a warning, not propagated as failures
3. **Run twice, intersect** — for high-severity findings (G001, G002, G005), the analysis is run twice. Only findings present in both runs are reported. This eliminates hallucinated gaps that appear in one run but not another

The cost of running twice is justified for high-severity findings — a false G005 (architectural contradiction) that fails CI is highly disruptive. Medium and low severity findings are single-run only.

---