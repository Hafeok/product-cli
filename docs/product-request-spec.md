# Product Request Specification

> Standalone reference for the Product request model.
> Extracted from ADR-032 in `product-adrs.md`.
> The request is the only write interface to the knowledge graph.

---

## Overview

All mutations to the knowledge graph go through a Product request. There are no
individual create, link, or status commands. The request model provides:

- **Atomicity** — a request either succeeds completely or fails completely. No partial graph states.
- **Cross-artifact validation** — constraints that span multiple artifacts (e.g. a dependency
  requires a governing ADR) are checked across the full request before any file is written.
- **Intent as data** — the full intent is expressed as a YAML file that can be inspected,
  saved, and re-validated independently of any write operation.

Two operations. One type field distinguishes them:

| Type | Use |
|---|---|
| `create` | New artifacts that do not exist yet |
| `change` | Mutations to existing artifacts |
| `create-and-change` | Both in one atomic operation |

---

## `type: create`

New artifacts. All IDs are assigned by Product on apply — never declared by the author.
Forward references (`ref:`) allow artifacts in the same request to reference each other
before IDs are known.

### Minimal create — one feature

```yaml
type: create
reason: "Add cluster health endpoint"
artifacts:
  - type: feature
    title: Cluster Health Endpoint
    phase: 2
    domains: [api, networking]
```

### Full create — feature with ADR, TC, and dependency

```yaml
type: create
reason: "Add rate limiting to the resource API"
artifacts:
  - type: feature
    ref: ft-rate-limiting
    title: Rate Limiting
    phase: 2
    domains: [api, security]
    adrs: [ref:adr-token-bucket, ref:adr-redis-choice]
    tests: [ref:tc-rate-limit]
    uses: [ref:dep-redis]

  - type: adr
    ref: adr-token-bucket
    title: Token bucket algorithm for rate limiting
    domains: [api]
    scope: domain
    features: [ref:ft-rate-limiting]

  - type: adr
    ref: adr-redis-choice
    title: Redis for rate limit state
    domains: [api]
    scope: domain
    governs: [ref:dep-redis]
    features: [ref:ft-rate-limiting]

  - type: tc
    ref: tc-rate-limit
    title: Rate limit enforced at 100 req/s
    tc-type: scenario
    validates:
      features: [ref:ft-rate-limiting]
      adrs: [ref:adr-token-bucket]

  - type: dep
    ref: dep-redis
    title: Redis
    dep-type: service
    version: ">=7"
    adrs: [ref:adr-redis-choice]
    interface:
      protocol: tcp
      port: 6379
      auth: password
      connection-string-env: REDIS_URL
    availability-check: "redis-cli ping"
```

After `product request apply`, all `ref:` values are replaced with real IDs (`FT-009`,
`ADR-031`, etc.) in every written file. Cross-links are bidirectional — if `ft-rate-limiting`
declares `adrs: [ref:adr-token-bucket]`, then `adr-token-bucket` gets `features: [FT-009]`
automatically.

### Artifact fields by type

**feature:**
```yaml
- type: feature
  ref: ft-name           # optional — only needed if referenced by other artifacts
  title: string          # required
  phase: integer         # required
  domains: []            # from [domains] vocabulary — E012 if unknown
  adrs: []               # ADR IDs or ref: values
  tests: []              # TC IDs or ref: values
  uses: []               # DEP IDs or ref: values
  depends-on: []         # FT IDs — implementation dependencies
  domains-acknowledged:  # required for each domain with no ADR coverage
    domain-name: "reasoning string — no empty string (E011)"
```

**adr:**
```yaml
- type: adr
  ref: adr-name          # optional
  title: string          # required
  domains: []
  scope: domain          # cross-cutting | domain | feature-specific
  features: []           # FT IDs or ref: values
  governs: []            # DEP IDs or ref: values — E013 if dep has no governing ADR
  supersedes: ~          # ADR ID this replaces
```

**tc:**
```yaml
- type: tc
  ref: tc-name           # optional
  title: string          # required
  tc-type: scenario      # scenario | invariant | chaos | exit-criteria | benchmark
  validates:
    features: []         # FT IDs or ref: values
    adrs: []             # ADR IDs or ref: values
  runner: ~              # cargo-test | bash | pytest | custom
  runner-args: []
  runner-timeout: 30s
  requires: []           # DEP IDs or [verify.prerequisites] key names
```

**dep:**
```yaml
- type: dep
  ref: dep-name          # optional
  title: string          # required
  dep-type: library      # library | service | api | tool | hardware | runtime
  version: string
  adrs: []               # governing ADR — required (E013 if absent)
  interface: ~           # optional: protocol, port, auth, env vars (service/api types)
  availability-check: ~  # shell command — null for libraries
  breaking-change-risk: low   # low | medium | high
```

### Forward reference rules

- A `ref:` value is a local name, scoped to the current request file. It has no meaning
  outside the request.
- Any artifact field that accepts an ID also accepts `ref:local-name`.
- Forward references are resolved in dependency order. Product builds a dependency graph
  of the request's artifacts, assigns IDs in topological order, then resolves all refs.
- A `ref:` that doesn't resolve to any artifact in the request is a validation error.

---

## `type: change`

Mutations to existing artifacts. Each change targets one existing artifact by its real ID
and declares one or more mutations.

### Minimal change — add a domain

```yaml
type: change
reason: "Add security domain after preflight W010"
changes:
  - target: FT-009
    mutations:
      - op: append
        field: domains
        value: security
```

### Change with acknowledgement

```yaml
type: change
reason: "Acknowledge security domain — no trust boundaries introduced"
changes:
  - target: FT-009
    mutations:
      - op: append
        field: domains
        value: security
      - op: set
        field: domains-acknowledged.security
        value: "Rate limiting is not an auth boundary — security handled by ADR-015"
```

### Multi-target change

```yaml
type: change
reason: "Link ADR-029 code quality to all phase 1 features"
changes:
  - target: FT-001
    mutations:
      - op: append
        field: adrs
        value: ADR-029

  - target: FT-002
    mutations:
      - op: append
        field: adrs
        value: ADR-029

  - target: ADR-029
    mutations:
      - op: append
        field: features
        value: FT-001
      - op: append
        field: features
        value: FT-002
```

### Body mutation (prose below front-matter)

```yaml
type: change
reason: "Fix typo in ADR-002 rationale"
changes:
  - target: ADR-002
    mutations:
      - op: set
        field: body
        value: |
          ## Context
          PiCloud requires distributed consensus for leader election...
          [corrected full body text]
```

Body mutations on accepted ADRs trigger E014 on the next `product graph check`
(content-hash mismatch). Resolve with `product adr accept ADR-002 --amend --reason "..."`.

### Mutation operations

| Op | Applies to | Behaviour |
|---|---|---|
| `set` | any scalar, string, nested field | Replace field value entirely |
| `append` | array fields | Add value — deduplicates, no error if already present |
| `remove` | array fields | Remove value — no error if not present |
| `delete` | optional fields | Remove the field from front-matter entirely |

**Dot-notation for nested fields:**
```yaml
# Sets domains-acknowledged.security in front-matter
field: domains-acknowledged.security
value: "reasoning string"

# Sets interface.port on a DEP
field: interface.port
value: 6379
```

---

## `type: create-and-change`

New artifacts plus mutations to existing ones. The `artifacts` section and `changes` section
are both present. Forward references from new artifacts can appear in change mutation values.

### Create a TC and link it to an existing feature

```yaml
type: create-and-change
reason: "Add exit criteria TC to FT-003"
artifacts:
  - type: tc
    ref: tc-rdf-restart
    title: RDF store survives restart
    tc-type: exit-criteria
    validates:
      features: [FT-003]
      adrs: [ADR-008]

changes:
  - target: FT-003
    mutations:
      - op: append
        field: tests
        value: ref:tc-rdf-restart   # resolved to real TC ID on apply
```

---

## Validation

All validation runs across the full request before any file is written. Every problem
is reported at once — not just the first one encountered.

### Within the request

| Rule | Error |
|---|---|
| `ref:` value not defined in request | E002 |
| DEP with no governing ADR in request or existing graph | E013 |
| Domain value not in `[domains]` vocabulary | E012 |
| `scope` not one of `cross-cutting / domain / feature-specific` | E006 |
| `tc-type` not a valid value | E006 |
| `dep-type` not a valid value | E006 |

### Against the existing graph

| Rule | Error |
|---|---|
| `target:` ID does not exist | E002 |
| `value:` ID (non-ref) does not exist and is not being created | E002 |
| `depends-on` creates a cycle | E003 |
| `supersedes` creates a cycle | E004 |

### Advisory (non-blocking)

| Rule | Output |
|---|---|
| New ADR has potential conflicts with existing ADRs | G005 advisory — reported, not blocking unless high severity |
| New DEP has `breaking-change-risk: high` | Printed as a note, not an error |

---

## Apply Behaviour

```
product request apply my-request.yaml
```

Steps:
1. Run full validation — exit 1 if any E-class finding, nothing written
2. Acquire advisory lock
3. Assign IDs in dependency order (topological sort of the request's artifact graph)
4. Resolve all `ref:` values to assigned IDs
5. Write new artifact files atomically (temp + rename + fsync)
6. Mutate changed artifact files atomically
7. Release lock
8. Run `product graph check` — report findings (files already written — this is a health check, not a gate)
9. Print summary

### Terminal output

```
product request apply rate-limiting-cr.yaml

  Validating...  ✓ clean

  Applying:
    FT-009  Rate Limiting                     [new feature]
    ADR-031 Token bucket algorithm            [new ADR]
    ADR-032 Redis for rate limit state        [new ADR]
    TC-050  Rate limit enforced at 100 req/s  [new TC]
    DEP-007 Redis                             [new dep]

  Graph check...  ✓ clean

  Done. 5 artifacts created.
  Run `git push --tags` after product verify FT-009.
```

### MCP output (`product_request_apply`)

```json
{
  "applied": true,
  "created": [
    { "ref": "ft-rate-limiting", "id": "FT-009",  "file": "docs/features/FT-009-rate-limiting.md" },
    { "ref": "adr-token-bucket", "id": "ADR-031", "file": "docs/adrs/ADR-031-token-bucket.md" },
    { "ref": "adr-redis-choice", "id": "ADR-032", "file": "docs/adrs/ADR-032-redis-rate-limit.md" },
    { "ref": "tc-rate-limit",    "id": "TC-050",  "file": "docs/tests/TC-050-rate-limit-100rps.md" },
    { "ref": "dep-redis",        "id": "DEP-007", "file": "docs/deps/DEP-007-redis.md" }
  ],
  "changed": [],
  "graph_check_clean": true
}
```

---

## Commands

```
product request create        # open $EDITOR with a create template
product request change        # open $EDITOR with a change template
product request validate FILE # validate without writing — shows all findings
product request apply FILE    # validate then write atomically
product request diff FILE     # show what would change, write nothing
product request draft         # list saved drafts in .product/requests/
```

Drafts live in `.product/requests/` (gitignored by default). A draft is just a file —
it has no special status. `product request apply` works on any YAML file, anywhere.

---

## MCP Tools

Two tools replace the entire individual write surface:

**`product_request_validate`**
```json
// Input
{ "request_yaml": "type: create\nreason: ...\n..." }

// Output
{
  "valid": false,
  "findings": [
    {
      "code": "E013",
      "severity": "error",
      "message": "DEP-007 has no governing ADR in request or existing graph",
      "location": "artifacts[4]"
    }
  ]
}
```

**`product_request_apply`**
```json
// Input
{ "request_yaml": "type: create\nreason: ...\n..." }

// Output — see Apply Behaviour above
```

The agent workflow is always:
1. Produce request YAML
2. Call `product_request_validate` — fix any findings
3. Call `product_request_apply` — receive assigned IDs
4. Continue with the real IDs (e.g. call `product_context FT-009`)

---

## Common Patterns

### Pattern: new feature from scratch

```yaml
type: create
reason: "Add workload health monitoring"
artifacts:
  - type: feature
    ref: ft-health
    title: Workload Health Monitoring
    phase: 3
    domains: [scheduling, observability]
    adrs: [ref:adr-health-checks]
    tests: [ref:tc-unhealthy-restarts]

  - type: adr
    ref: adr-health-checks
    title: HTTP liveness and readiness probes
    domains: [scheduling]
    scope: domain
    features: [ref:ft-health]

  - type: tc
    ref: tc-unhealthy-restarts
    title: Unhealthy container restarted within 30s
    tc-type: scenario
    validates:
      features: [ref:ft-health]
      adrs: [ref:adr-health-checks]
```

### Pattern: link an existing ADR to an existing feature

```yaml
type: change
reason: "ADR-029 code quality applies to FT-003"
changes:
  - target: FT-003
    mutations:
      - op: append
        field: adrs
        value: ADR-029
  - target: ADR-029
    mutations:
      - op: append
        field: features
        value: FT-003
```

### Pattern: acknowledge a domain gap

```yaml
type: change
reason: "Acknowledge IAM domain — FT-007 has no auth boundary"
changes:
  - target: FT-007
    mutations:
      - op: append
        field: domains
        value: iam
      - op: set
        field: domains-acknowledged.iam
        value: "RDF store has no direct IAM surface — access is mediated by the platform layer"
```

### Pattern: deprecate a dependency

```yaml
type: change
reason: "Mark DEP-005 deprecated — migrating to embedded store"
changes:
  - target: DEP-005
    mutations:
      - op: set
        field: status
        value: deprecated
```

### Pattern: add a TC runner after infrastructure is available

```yaml
type: change
reason: "Configure TC-042 runner now that test cluster is available"
changes:
  - target: TC-042
    mutations:
      - op: set
        field: runner
        value: bash
      - op: set
        field: runner-args
        value: ["scripts/test-harness/event-store.sh"]
      - op: set
        field: runner-timeout
        value: 180s
```

### Pattern: supersede an ADR

```yaml
type: create-and-change
reason: "Replace ADR-006 with updated context bundle design"
artifacts:
  - type: adr
    ref: adr-new-bundle
    title: Context bundle with dependency section
    domains: [api]
    scope: cross-cutting
    supersedes: ADR-006
    features: [FT-001, FT-002, FT-003]

changes:
  - target: ADR-006
    mutations:
      - op: set
        field: status
        value: superseded
      - op: set
        field: superseded-by
        value: ref:adr-new-bundle
```

---

## Invariants

- A failed `product request apply` leaves **zero files changed**. Verified by checksumming
  all artifact files before and after a failing apply.
- A successful apply followed by `product graph check` always exits 0 or 2 — never 1.
  Exit 1 after a successful apply is a Product bug.
- `product request validate` never writes to disk under any circumstances.
- `append` is idempotent — applying the same append twice produces the same result as
  applying it once.
