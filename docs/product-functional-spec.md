# Product Functional Specification Section

> Standalone reference for the Functional Specification section on feature artifacts.
> Extends the feature body format — no new artifact type.
> Purpose: give LLM agents a complete behavioural contract to implement against.
>
> New feature body structure, new config section, new warning code: W030.

---

## Overview

An LLM reading a feature context bundle needs more than a capability title and
a list of decisions — it needs a precise, structural description of what the
feature does, so that "implement FT-009" produces a result that matches what
the team expects without guessing.

The Functional Specification section is that description. It sits in the body
of the feature artifact, below the Description, with a fixed set of subsections.
It is:

- **Part of the feature**, not a separate artifact — always in the context bundle
- **Structural**, not free prose — completeness is checkable
- **The contract** — inputs, outputs, state, behaviour, invariants, errors, boundaries
- **Bounded** — includes an "Out of scope" subsection to prevent LLM scope creep

The TCs linked to the feature verify specific points on this contract. The
functional spec defines the contract the TCs are points on.

---

## Why this belongs on the feature

Three alternatives were considered:

**New SPEC-XXX artifact type** — rejected. Creates an extra graph node the
agent must traverse. Introduces "ADR vs SPEC" confusion about where decisions
live. Context bundles would need to expand to include SPEC artifacts.
The feature is where agents already look first — the spec belongs there.

**Spec as a set of exit-criteria TCs** — rejected. TCs verify instances; a
functional spec defines the space of correct behaviour. A spec expressed as
a hundred small TCs is harder for an agent to reason about than as one
structured document with TCs pointing to specific clauses.

**Structured section on the feature** — accepted. The feature body already
carries prose description. Extending it with a fixed-structure Functional
Specification section uses existing infrastructure, keeps related information
together, and appears in every context bundle for that feature without any
extra graph traversal.

---

## Feature Body Structure

```markdown
---
id: FT-009
title: Rate Limiting
phase: 2
status: planned
domains: [api, security]
adrs: [ADR-031]
tests: [TC-050, TC-051, TC-052]
uses: [DEP-007]
---

## Description

The API enforces per-client rate limiting to prevent abuse and ensure
fair resource allocation across tenants.

## Functional Specification

### Inputs
  ...

### Outputs
  ...

### State
  ...

### Behaviour
  ...

### Invariants
  ...

### Error handling
  ...

### Boundaries
  ...

## Out of scope

  ...
```

The structure is fixed. Every feature with `phase >= 1` (i.e. not a stub)
should have all sections. Empty sections are valid if the concept genuinely
doesn't apply — "This feature is stateless" in the State section, for example.

---

## Subsection Definitions

Each subsection answers a specific question. Together they define the complete
behavioural contract.

### Inputs

**What does this feature accept from the outside?**

Every input the feature responds to. HTTP requests with their headers, body
schema, and required/optional fields. CLI arguments. Message queue payloads.
Configuration values. Environment variables.

Include:
- Names (header names, field names, CLI flag names — exactly as they appear)
- Types and value ranges (non-empty string, integer in 1..1000, ISO 8601 date)
- Required vs optional
- Defaults when optional

```markdown
### Inputs
  - HTTP request with `X-Client-Id` header
      required, non-empty string, max 128 characters
  - Optional `X-Request-Priority` header
      values: "normal" | "high"
      default: "normal"
  - Request body per downstream service requirements (passed through)
```

### Outputs

**What does this feature produce?**

Every output path, including success and failure. Status codes, response
bodies, headers, log entries, metrics emitted, side effects.

Include:
- Exact status codes and when each applies
- Response body schema
- Header names (exactly as set)
- Non-HTTP outputs: metrics, logs, events emitted

```markdown
### Outputs
  - On allow: HTTP 200, response body from upstream unchanged
      Headers: X-RateLimit-Remaining: <tokens remaining after decrement>
  - On deny: HTTP 429
      Body: {"error": "rate_limit_exceeded", "retry_after": <seconds>}
      Headers:
        Retry-After: <seconds until tokens available>
        X-RateLimit-Limit: <capacity>
        X-RateLimit-Remaining: 0
  - On missing X-Client-Id: HTTP 400
      Body: {"error": "missing_client_id"}
  - Metric: rate_limit.decisions{outcome=allow|deny, client_id=...} counter
```

### State

**What does this feature remember between requests?**

Stateful features declare their state shape and transitions. Stateless features
say "stateless" — this is meaningful, not a placeholder.

Include:
- Data structures held in memory or storage
- Fields and their types
- Lifetime (request-scoped, session-scoped, persistent)

```markdown
### State
  Per-client token bucket, held in memory:
    - capacity: integer, from [rate-limit].capacity config (default 100)
    - refill_rate: tokens/second, from config (default 100)
    - tokens: float in [0, capacity]
    - last_refill: monotonic timestamp

  Buckets are created on first request from a client, evicted after 1 hour idle.
  State is not persisted across process restarts.
```

### Behaviour

**What does this feature do, given its inputs and state?**

Numbered or ordered step description of the core algorithm. Precise enough to
implement from. Avoid prose paragraphs — use structured steps.

```markdown
### Behaviour

  On incoming request:
    1. If X-Client-Id missing or empty → return 400 response
    2. Retrieve bucket for client_id, or initialize with tokens = capacity
    3. Compute elapsed = now - bucket.last_refill
    4. bucket.tokens = min(capacity, bucket.tokens + elapsed * refill_rate)
    5. bucket.last_refill = now
    6. If bucket.tokens >= 1:
         bucket.tokens -= 1
         Forward request upstream
         Return upstream response with X-RateLimit-Remaining = floor(bucket.tokens)
    7. Else:
         retry_after = ceil((1 - bucket.tokens) / refill_rate)
         Return 429 response with Retry-After = retry_after
```

### Invariants

**What must be true at all times?**

Properties the implementation must maintain. Tested by `invariant` and
`chaos` TCs. Use `⟦Γ:Invariants⟧` formal blocks if the team uses AISP —
but natural language is acceptable and preferred when formal notation
would add friction without adding precision.

```markdown
### Invariants
  - Token count per bucket ∈ [0, capacity] at all times
  - Refill is purely additive — tokens never decrease from refill operations
  - Two concurrent requests from the same client are strictly serialised:
    the decrement for request N completes before the refill for request N+1 begins
  - On process restart, all buckets reinitialize to full (no persistence)
```

### Error handling

**What happens when things go wrong, and how is that signalled?**

Every failure mode and its response. Distinct from "Outputs" because this is
about unexpected conditions, not normal deny-path outputs.

Include:
- Downstream failures (timeout, 5xx, connection refused)
- Internal failures (config missing, state corruption)
- Failure policy: fail open, fail closed, retry, etc.

```markdown
### Error handling
  - Upstream timeout (>30s):
      Return 504, bucket state unchanged (token already consumed on the way in)
      Log: rate_limit.upstream_timeout{client_id=...}
  - Upstream 5xx:
      Return upstream status unchanged, bucket state unchanged
  - Internal error in rate limit logic:
      Fail open — forward request upstream anyway
      Return 503 if upstream also fails
      Log: rate_limit.internal_error with full error context
      Emit metric: rate_limit.errors{type=...} counter
```

### Boundaries

**Edge cases and boundary conditions — what happens at the extremes?**

This is the subsection that prevents LLMs from implementing assumptions that
happen to work for common cases but fail at edges. Be explicit about every
boundary the implementer might otherwise have to guess.

```markdown
### Boundaries
  - First request from new client: bucket initialized to full (capacity tokens)
  - Clock adjustment backward (negative elapsed):
      Clamp elapsed to 0 — no token creation from negative time
      Update last_refill to current time
  - Clock adjustment forward (large positive elapsed):
      Refill is bounded by capacity — tokens saturate, no overflow
  - Client sends 1000 requests simultaneously:
      All serialised through the bucket mutex
      First 100 succeed (assuming bucket was full), rest return 429
  - Bucket eviction after 1 hour idle:
      Next request from that client re-initializes to full
      This is intentional — rate limits reset after long idle periods
```

### Out of scope

**What this feature explicitly does not do.**

For LLM consumption, this is as important as everything above. LLMs pattern-
match helpfully — given "rate limiting" they may implement endpoint-specific
limits, allow-lists, and distributed state unless told not to. Be explicit.

```markdown
## Out of scope
  - Per-endpoint rate limits (future: FT-014)
  - Distributed rate limit state across multiple API nodes (future: FT-018)
  - Client allow-list / exemptions (future: FT-015)
  - Rate limit configuration per-client (all clients use the same limits)
  - Persistence of bucket state across restarts
  - Metrics aggregation across nodes (each node reports independently)
```

Out of scope items may reference future features when applicable. This creates
a soft link — not a graph edge, but a pointer the LLM and reviewers can follow.

---

## Structural Completeness Check

`product graph check` validates that feature bodies contain the required
sections. Configured in `product.toml`:

```toml
[features]
# Required top-level sections in the feature body
required-sections = ["Description", "Functional Specification", "Out of scope"]

# Required subsections under Functional Specification
functional-spec-subsections = [
  "Inputs",
  "Outputs",
  "State",
  "Behaviour",
  "Invariants",
  "Error handling",
  "Boundaries"
]

# Features below this phase are exempt (typically stubs from migration)
required-from-phase = 1
```

### W030 — Missing required section

```
warning[W030]: feature body missing required section
  FT-009: Rate Limiting

  Missing sections:
    - Functional Specification > Behaviour
    - Functional Specification > Boundaries
    - Out of scope

  Add with: product request change, op: set, field: body
```

W030 is advisory by default. Promotable to an error via config:

```toml
[features]
completeness-severity = "warning"   # warning | error
```

When set to `error`, W030 becomes E-class and blocks request apply for
features changing status from `planned` to `in-progress`. Teams that want
functional specs before implementation begins opt in via this flag.

---

## Empty Subsections Are Valid

A feature that is genuinely stateless has:

```markdown
### State
  Stateless. No data is retained between requests.
```

A feature with no error handling beyond basic validation has:

```markdown
### Error handling
  No custom error handling. Input validation failures return 400 per standard
  API conventions.
```

Empty-meaning entries satisfy W030. What does not satisfy W030 is an absent
section or a section with only whitespace.

---

## Context Bundle Integration

The Functional Specification appears in context bundles wherever the feature
body appears. `product context FT-009 --depth 2` produces the full bundle
including the feature's complete body — which now includes the functional spec.

No new assembly logic needed. The context bundle already includes feature
bodies. The spec is part of the body.

This is the central design benefit: no new node to traverse, no extra link,
no new bundle shape. An LLM calling `product_context` via MCP gets the full
implementation contract in the response it already requested.

---

## Relationship to TCs

TCs verify specific points on the functional specification. They are not
replacements for it.

Example for FT-009:

```yaml
---
id: TC-050
title: Rate limit enforced at exact capacity
type: scenario
level: integration
validates:
  features: [FT-009]
  adrs: [ADR-031]
---

Verifies Functional Specification / Behaviour step 7:
  When tokens < 1, request returns 429 with Retry-After header.
  When tokens >= 1, request is forwarded and tokens decrement.

Test:
  Send 101 requests in rapid succession from client_id=test-001.
  Assert requests 1-100 succeed.
  Assert request 101 returns 429.
  Assert request 101 Retry-After header ≈ 10ms (1 / refill_rate).
```

The TC body can reference the spec subsection it validates. No enforcement —
it's a convention that helps reviewers connect the test to the contract.

---

## Relationship to ADRs

ADRs explain why decisions were made. The functional spec explains what
behaviour those decisions produce. They're distinct:

**ADR-031 (Token Bucket Algorithm):**
```
Decision: Use token bucket algorithm over fixed window counter.
Rationale: Token bucket allows short bursts while enforcing long-term rate.
  Fixed window has edge behaviour at window boundaries that token bucket avoids.
Rejected: Leaky bucket (harder to reason about from client perspective),
  fixed window (burst edge cases).
```

**FT-009 Functional Specification (excerpt):**
```
Behaviour:
  ...
  3. Compute elapsed = now - bucket.last_refill
  4. bucket.tokens = min(capacity, bucket.tokens + elapsed * refill_rate)
  ...
```

The ADR chose the algorithm. The spec describes what the chosen algorithm
does when the system receives a request. An LLM implementing needs both.

---

## Migration

Existing features may not have a full functional specification — they were
written before this structure existed. W030 fires on each one.

To migrate, author a change request for each feature adding the missing
sections. The `product request new change` builder supports editing the body
field via `product request edit` — open the feature file in `$EDITOR`, add
the sections, save.

For bulk migration:

```bash
# List features missing functional specs
product graph check --filter W030

# Get LLM help writing them
product context FT-009 | claude "write a Functional Specification for this feature"
```

The spec is structural enough that an LLM can draft one from the existing
ADRs, TCs, and feature description — and a human reviews and commits it.

---

## Session Tests

```
# Section detection
ST-340  feature-body-parser-recognizes-functional-specification-section
ST-341  feature-body-parser-recognizes-all-subsections
ST-342  w030-fires-when-required-section-missing
ST-343  w030-fires-when-required-subsection-missing
ST-344  w030-clear-when-all-sections-present
ST-345  w030-respects-required-from-phase

# Completeness severity
ST-346  completeness-severity-warning-w030-is-w-class
ST-347  completeness-severity-error-w030-becomes-e-class
ST-348  completeness-error-blocks-in-progress-transition

# Empty sections
ST-349  empty-meaning-section-satisfies-w030
ST-350  whitespace-only-section-emits-w030
ST-351  absent-section-emits-w030

# Context bundle
ST-352  context-bundle-includes-full-functional-spec
ST-353  context-bundle-preserves-subsection-structure

# Config
ST-354  required-sections-configurable
ST-355  functional-spec-subsections-configurable
```

---

## Invariants

- The Functional Specification section lives in the feature body. There is
  no parallel SPEC-XXX artifact. The feature artifact is authoritative.
- Section structure is fixed per `product.toml` configuration. The parser
  recognises sections by their exact H2 and H3 headings.
- W030 is computed structurally from the parsed feature body. No LLM is
  involved in checking completeness.
- Context bundles include the full feature body including all functional spec
  subsections. The bundle assembly does not truncate or summarise.
- The "Out of scope" section is at the top level (H2), not inside Functional
  Specification. It applies to the whole feature, not just its behaviour.
- Empty sections with explicit "stateless" / "no custom error handling" text
  satisfy W030. Absence of the section does not.
- The spec does not replace DOCs. A DOC artifact of `type: reference` may
  still document the feature for external users — typically generated from
  or informed by the functional spec, but separate.
