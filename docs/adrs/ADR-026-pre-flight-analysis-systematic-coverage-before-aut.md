---
id: ADR-026
title: Pre-flight Analysis — Systematic Coverage Before Authoring
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** ADR-025 introduces domain classification and cross-cutting scope. This creates the data needed for systematic coverage checking. The question is when and how that check runs.

Two options: passive (surface gaps after the feature is authored, via `graph check` warnings) or active (surface gaps before authoring begins, via a dedicated pre-flight command that blocks until acknowledged).

Passive is insufficient. By the time a feature is authored, the author has invested effort in a spec that may need significant revision to address domain gaps. The cost of late discovery is high.

Active pre-flight is the right model. It runs before the authoring session starts, presents the specific gaps, and requires either linking or acknowledging each one before proceeding. The authoring agent cannot begin until the coverage check is clean.

**Decision:** `product preflight FT-XXX` is a mandatory first step in the `product author feature` session and the `product implement FT-XXX` pipeline. It analyses the feature against the domain taxonomy and cross-cutting ADR set, presents a structured coverage report, and requires each gap to be resolved (linked or acknowledged) before the session or pipeline continues. Preflight results are cached for the duration of the session — subsequent runs within the same session skip the LLM calls.

---

### Preflight Report Format

```
product preflight FT-009

Pre-flight analysis: FT-009 — Rate Limiting
Feature domains: networking, api

━━━ Cross-Cutting ADRs (must acknowledge all) ━━━━━━━━━━━━━━

  ✓  ADR-001  Rust as implementation language          [linked]
  ✓  ADR-013  Error model and diagnostics              [linked]
  ✓  ADR-015  File write safety                        [linked]
  ✗  ADR-038  Observability requirements               [not linked, not acknowledged]
  ✗  ADR-040  CLI output conventions                   [not linked, not acknowledged]

━━━ Domain Coverage ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  networking  ✓  ADR-004 (linked), ADR-006 (linked)
  api         ✓  ADR-009 (linked), ADR-012 (linked)
  security    ✗  no coverage — 4 ADRs in domain, none linked or acknowledged
  iam         ✗  no coverage — 3 ADRs in domain, none linked or acknowledged
  storage     ~  ADR-007 (linked) — review: does rate limiting touch storage?

━━━ Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  2 cross-cutting ADRs unacknowledged
  2 domains with no coverage
  1 domain flagged for review

To resolve:
  product feature link FT-009 --adr ADR-038
  product feature acknowledge FT-009 --domain security --reason "..."
  product feature acknowledge FT-009 --domain iam --reason "..."
  product feature acknowledge FT-009 --adr ADR-040 --reason "..."
```

---

### Resolution Commands

```bash
# Link an ADR (adds to feature's adrs list)
product feature link FT-009 --adr ADR-038

# Acknowledge a domain gap with reasoning
product feature acknowledge FT-009 --domain security \
  --reason "rate limiting operates at resource API layer, no trust boundaries introduced"

# Acknowledge a cross-cutting ADR with reasoning
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "rate limiting has no special output requirements beyond standard error model"

# Acknowledge all gaps at once with a shared reason (use carefully)
product feature acknowledge FT-009 --all-domains \
  --reason "reviewed all domains, see individual notes in ADR-021"
```

All acknowledgement commands mutate feature front-matter atomically (ADR-015) and re-run preflight validation to confirm the gap is closed.

---

### Preflight in the `product author` Session

When `product author feature` is invoked, the first action before any user message is processed:

1. Run `product preflight FT-XXX`
2. If preflight is clean — proceed to authoring
3. If preflight has gaps — present the report to Claude via the MCP tool surface
4. Claude reads the unacknowledged cross-cutting ADRs and domain ADRs
5. For each gap, Claude either:
   - Calls `product_feature_link` to add the ADR (if it's clearly relevant)
   - Calls `product_feature_acknowledge` with a specific reason (if it's clearly not applicable)
   - Asks the user to make the call (if intent is ambiguous)
6. Re-runs preflight — repeats until clean
7. Only then begins writing feature content

This makes the domain coverage check part of the authoring session's natural flow rather than a separate gate the user must remember to run.

---

### Preflight in `product implement`

`product implement FT-XXX` already has a gap gate (ADR-021, Step 1). Preflight is inserted as Step 0, before the gap gate:

```
product implement FT-009

  Step 0 — Pre-flight check
  ✗ 2 cross-cutting ADRs unacknowledged (ADR-038, ADR-040)
  ✗ 2 domains with no coverage (security, iam)

  Implementation blocked. Run: product preflight FT-009
  Then re-run: product implement FT-009
```

Preflight failures always block `product implement`. Unlike the gap gate (which can be suppressed), preflight coverage gaps cannot be bypassed — they must be resolved or acknowledged.

---

### Coverage Matrix: `product graph coverage`

The portfolio-level view of feature × domain coverage:

```
product graph coverage

                    sec  stor  cons  net  obs  err  iam  sched  api
FT-001 Cluster       ✓    ✓     ✓    ✓    ✓    ✓    ✓    ✓     ✓
FT-002 Products      ✓    ✓     ·    ✓    ✓    ✓    ✓    ·     ✓
FT-003 RDF Store     ~    ✓     ·    ·    ✓    ✓    ~    ·     ✓
FT-004 IAM           ✓    ·     ·    ✓    ✓    ✓    ✓    ·     ✓
FT-009 Rate Limit    ✗    ✗     ·    ✓    ✗    ✗    ✗    ·     ✓

Legend:  ✓ covered (linked)   ~ acknowledged   · not applicable   ✗ gap
```

`·` (not applicable) appears when: the feature does not declare the domain in its `domains` field AND no cross-cutting ADRs exist for that domain. The domain genuinely doesn't apply.

`~` (acknowledged) appears when the domain is acknowledged with a reason but not linked to an ADR.

`product graph coverage --domain security` filters to show only the security column with full ADR details for each feature.

`product graph coverage --format json` produces machine-readable output for CI reporting.

---

### Preflight Caching

Preflight analysis involves graph traversal but no LLM calls. It runs in < 100ms on any repository up to 200 features. No caching is needed — each `product preflight` invocation reads the graph fresh (consistent with ADR-003: no persistent graph store).

---

**Rationale:**
- Pre-flight as Step 0 in `product implement` is the right enforcement point. The implementation agent should never receive a context bundle that has known domain gaps — it would implement a feature without knowing it's missing security consideration. The gate prevents this.
- Non-bypassability distinguishes pre-flight from gap analysis. Gap findings can be suppressed with a reason. Domain coverage gaps can be acknowledged with a reason. But acknowledgement is not suppression — it is an explicit statement of intent. The distinction matters: suppressing a gap silences the finding; acknowledging a domain gap documents a conscious decision.
- The coverage matrix is the most valuable output for engineering leadership and code review. It makes architectural blind spots visible at a glance. A feature with six domain gaps in the `✗` column is not ready for implementation regardless of whether its test criteria are written.
- Limiting domain ADR inclusion in preflight to top-2 by centrality is inherited from ADR-025. The same reasoning applies: centrality filters to the foundational decisions without requiring the author to read all 15 storage ADRs when adding a feature that touches storage.

**Rejected alternatives:**
- **Passive gap surfacing only (W010/W011 in `graph check`)** — discovered after authoring. Cost of late discovery is high. Rejected.
- **Mandatory ADR links for all domains** — cannot acknowledge a domain as not-applicable. Every feature touching `networking` would need to link all networking ADRs or get a hard error. Too rigid — not every feature has security implications, and forcing links creates meaningless associations. Rejected.
- **LLM-assisted domain assignment** — let the LLM suggest which domains a feature touches based on its content. Reduces manual tagging burden. Rejected for v1: LLM-suggested domains introduce non-determinism into what should be an explicit author decision. Can be added as `product preflight --suggest-domains` in a future version.
- **Single acknowledgement for all gaps** — one `--acknowledge-all` flag that silences every gap without requiring per-domain reasoning. Rejected: this is equivalent to suppression with no audit trail. The per-domain reasoning is the point.