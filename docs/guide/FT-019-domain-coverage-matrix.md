## Overview

Domain Coverage Matrix provides a portfolio-level view of how thoroughly each feature addresses the project's architectural concern domains. Every project defines a vocabulary of domains (security, storage, networking, etc.) in `product.toml`. ADRs declare which domains they govern and whether they are cross-cutting (apply to every feature) or domain-scoped. Features declare which domains they touch. The coverage matrix cross-references these declarations to surface gaps — features that claim to touch a domain but have no linked ADR or explicit acknowledgement. Two commands expose this data: `product graph coverage` for the full feature × domain matrix, and `product preflight FT-XXX` for a single-feature deep dive with actionable resolution commands. Pre-flight is enforced as Step 0 of `product implement`, blocking implementation until all domain gaps are resolved or acknowledged.

## Tutorial

### Step 1: Check Your Domain Vocabulary

Domains are declared in `product.toml`. Open the file to see what domains your project defines:

```toml
[domains]
security        = "Authentication, authorisation, secrets, trust boundaries"
storage         = "Persistence, durability, volume, block devices, backup"
networking      = "mDNS, mTLS, DNS, service discovery, port allocation"
error-handling  = "Error model, diagnostics, exit codes, panics, recovery"
observability   = "OTel, metrics, tracing, logging, telemetry"
```

Each domain is a short name with a one-sentence description. This vocabulary is project-specific — add or remove domains as your system evolves.

### Step 2: See the Coverage Matrix

Run the coverage matrix to get a bird's-eye view of your entire project:

```bash
product graph coverage
```

You'll see output like:

```
                    sec  stor  cons  net  obs  err  iam  sched  api  data
FT-001 Cluster       ✓    ✓     ✓    ✓    ✓    ✓    ✓    ✓     ✓    ✓
FT-002 Products      ✓    ✓     ·    ✓    ✓    ✓    ✓    ·     ✓    ·
FT-003 RDF Store     ~    ✓     ·    ·    ✓    ✓    ~    ·     ✓    ✓
FT-009 Rate Limit    ✗    ✗     ·    ✓    ✗    ✗    ✗    ·     ✓    ·

Legend:
  ✓  covered      — feature has a linked ADR in this domain
  ~  acknowledged — domain acknowledged with explicit reasoning, no linked ADR
  ·  not declared — feature does not declare this domain (may still apply)
  ✗  gap          — feature declares domain but has no coverage
```

Any `✗` in the matrix is a gap that needs attention before implementation.

### Step 3: Run Pre-flight on a Single Feature

Pick a feature with gaps and run pre-flight to get specific details:

```bash
product preflight FT-009
```

The report shows three sections: cross-cutting ADRs that must be acknowledged, domain coverage status, and resolution commands you can copy-paste.

### Step 4: Resolve a Domain Gap by Linking an ADR

If the feature genuinely relates to an ADR, link it:

```bash
product feature link FT-009 --adr ADR-038
```

### Step 5: Resolve a Domain Gap by Acknowledging It

If the domain applies but no ADR link is needed, acknowledge it with a reason:

```bash
product feature acknowledge FT-009 --domain security \
  --reason "Rate limiting operates at the Resource API layer. No new trust boundaries introduced."
```

The reason is mandatory — an empty reason produces validation error E011.

### Step 6: Confirm Pre-flight Is Clean

Re-run pre-flight to verify all gaps are resolved:

```bash
product preflight FT-009
```

When clean, the command exits 0 and prints "Pre-flight clean."

### Step 7: Proceed to Implementation

With pre-flight clean, `product implement` will proceed past Step 0:

```bash
product implement FT-009
```

## How-to Guide

### View the Full Coverage Matrix

```bash
product graph coverage
```

Shows every feature against every domain. Scan for `✗` symbols to find gaps.

### Filter the Matrix to One Domain

```bash
product graph coverage --domain security
```

Shows only the security column with full ADR details per feature.

### Export Coverage as JSON for CI

```bash
product graph coverage --format json
```

Produces machine-readable output with a `features` array, each containing a `domains` map with coverage status values.

### Run Pre-flight for a Feature

```bash
product preflight FT-009
```

Returns exit code 0 if clean, exit code 1 if gaps exist. The report includes copy-paste resolution commands.

### Link an ADR to a Feature

```bash
product feature link FT-009 --adr ADR-038
```

Adds the ADR to the feature's `adrs` list in front-matter. The write is atomic (ADR-015). Pre-flight re-validates after the mutation.

### Acknowledge a Domain Gap

```bash
product feature acknowledge FT-009 --domain security \
  --reason "No trust boundaries introduced. IAM enforces access upstream."
```

Adds an entry to `domains-acknowledged` in the feature's front-matter. The reason is mandatory.

### Acknowledge a Cross-Cutting ADR

```bash
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "No special output requirements beyond the standard error model."
```

### Acknowledge All Domain Gaps at Once

```bash
product feature acknowledge FT-009 --all-domains \
  --reason "Reviewed all domains, see individual notes in ADR-021"
```

Use sparingly — per-domain reasoning is preferred.

### Check for Domain Validation Warnings

```bash
product graph check
```

Reports W010 (unacknowledged cross-cutting ADR) and W011 (domain gap without acknowledgement) alongside other graph health checks.

## Reference

### Commands

| Command | Description |
|---|---|
| `product graph coverage` | Full feature × domain coverage matrix |
| `product graph coverage --domain <name>` | Single-domain column with ADR details |
| `product graph coverage --format json` | JSON output for CI integration |
| `product preflight FT-XXX` | Single-feature pre-flight coverage report |
| `product feature link FT-XXX --adr ADR-XXX` | Link an ADR to a feature |
| `product feature acknowledge FT-XXX --domain <name> --reason "..."` | Acknowledge a domain gap |
| `product feature acknowledge FT-XXX --adr ADR-XXX --reason "..."` | Acknowledge a cross-cutting ADR |
| `product feature acknowledge FT-XXX --all-domains --reason "..."` | Acknowledge all domain gaps |

### Coverage Symbols

| Symbol | Meaning | Condition |
|---|---|---|
| `✓` | Covered | Feature has a linked ADR in this domain |
| `~` | Acknowledged | Domain acknowledged with explicit reasoning, no linked ADR |
| `·` | Not declared | Feature does not declare this domain and no cross-cutting ADRs exist for it |
| `✗` | Gap | Feature declares the domain but has no linked ADR and no acknowledgement |

### ADR Front-Matter Fields

| Field | Type | Description |
|---|---|---|
| `domains` | list of strings | Concern domains this ADR governs (must be in `product.toml` vocabulary) |
| `scope` | string | `cross-cutting`, `domain`, or `feature-specific` (default: `feature-specific`) |

### Feature Front-Matter Fields

| Field | Type | Description |
|---|---|---|
| `domains` | list of strings | Concern domains this feature touches |
| `domains-acknowledged` | map of string to string | Domain name to reasoning for explicit acknowledgement without ADR link |

### Configuration (`product.toml`)

```toml
[domains]
security    = "Authentication, authorisation, secrets, trust boundaries"
storage     = "Persistence, durability, volume, block devices, backup"
networking  = "mDNS, mTLS, DNS, service discovery, port allocation"
```

Each key is a domain name. The value is a one-sentence description. Features and ADRs reference domains by these keys. Using an undefined domain produces validation error E012.

### Validation Errors and Warnings

| Code | Severity | Condition |
|---|---|---|
| E011 | Error | `domains-acknowledged` entry with empty or whitespace-only reason |
| E012 | Error | Feature or ADR references a domain not defined in `product.toml` |
| W010 | Warning | Cross-cutting ADR not linked to or acknowledged by a feature |
| W011 | Warning | Feature declares a domain with domain-scoped ADRs but has no coverage |

### Exit Codes

- `product preflight`: exits 0 when clean, exits 1 when gaps exist.
- `product implement`: exits 1 at Step 0 if pre-flight has unresolved gaps.

### Context Bundle Ordering

When assembling a context bundle (`product context FT-XXX`), cross-cutting and domain ADRs are included automatically:

1. Feature content
2. Cross-cutting ADRs (all, ordered by betweenness centrality)
3. Domain ADRs (top-2 by centrality per declared domain)
4. Feature-linked ADRs (direct links, by centrality)
5. Test criteria

Cross-cutting ADRs are included even without explicit graph links.

### JSON Output Schema

`product graph coverage --format json` produces:

```json
{
  "features": [
    {
      "id": "FT-009",
      "title": "Rate Limiting",
      "domains": {
        "security": "gap",
        "networking": "covered",
        "api": "covered",
        "storage": "acknowledged"
      }
    }
  ]
}
```

## Explanation

### Why Domains Exist

At scale (100+ ADRs), the knowledge graph has a discovery problem. ADRs are structurally identical nodes — there is no way to ask "show me all security ADRs" without already knowing which ones they are. The domain vocabulary solves this by classifying ADRs into concern areas. This classification is project-specific, not a universal taxonomy. See ADR-025 for the full rationale.

### Cross-Cutting vs. Domain Scope

Some ADRs apply to every feature unconditionally. ADR-013 (error model) governs how every component surfaces errors. These are `scope: cross-cutting` — the system enforces that every feature either links them or explicitly acknowledges them with a reason.

Domain-scoped ADRs apply only to features that declare the relevant domain. A feature that doesn't touch storage has no obligation to address storage ADRs. The `scope: domain` designation makes this conditional enforcement possible.

Features that have neither `domains` nor explicit scope default to `feature-specific`, preserving backward compatibility with existing ADRs.

### Why Acknowledgements Require Reasoning

An acknowledgement without reasoning is indistinguishable from a checkbox ticked to silence a warning. The mandatory reason serves two purposes: it proves the author actually considered the domain, and it becomes documentation for future readers who need to understand why a domain was scoped out. This is the critical design distinction from simple suppression — see ADR-025.

### Why Pre-flight Is Non-Bypassable

Gap analysis findings (from `product gap check`) can be suppressed. Pre-flight coverage gaps cannot — they must be resolved or acknowledged. The distinction is intentional: suppression silences a finding, while acknowledgement documents a conscious decision. An implementation agent should never receive a context bundle with known domain gaps, because it would implement a feature without understanding why certain domains were excluded. Pre-flight as Step 0 in `product implement` prevents this. See ADR-026.

### Why Domain ADRs Are Limited to Top-2 by Centrality

A domain like security might have 15 ADRs. Including all of them in every context bundle for every feature that touches security would cause context explosion. Instead, only the top-2 ADRs by betweenness centrality are included — these are the most foundational decisions in the domain, the ones that govern the others. This filtering strategy is defined in ADR-025 and applies to both context bundles and pre-flight reports.

### Relationship to Other Features

- **Context Bundles**: Cross-cutting and domain ADRs are automatically included in bundles assembled by `product context`. The bundle ordering (cross-cutting → domain → feature-linked → test criteria) ensures the implementation agent sees governance constraints first.
- **Graph Health** (`product graph check`): Domain coverage warnings (W010, W011) are part of the standard health check suite. Pre-flight is the active enforcement; graph check is the passive audit.
- **Implementation Pipeline** (`product implement`): Pre-flight is Step 0, running before gap analysis (Step 1) and context assembly. If pre-flight fails, no context bundle is assembled and no agent is invoked.
- **Authoring Sessions** (`product author feature`): Pre-flight runs automatically as the first action before any user interaction. The authoring agent resolves gaps interactively before writing feature content.
