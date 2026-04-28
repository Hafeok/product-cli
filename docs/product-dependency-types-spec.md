# Product Dependency Types Specification

> Standalone reference for the dependency type model.
> Two orthogonal fields: `dep-type` (what it is) and `dep-relationship` (how you depend on it).
>
> Refines the original DEP artifact schema in ADR-007.
> New field: `dep-relationship`
> Refined field: `dep-type`

---

## The Distinction

Two questions about every external dependency that the original schema
collapsed into one:

**What kind of thing is it?**
A code library? A network service? A piece of hardware?

**How do you depend on it?**
Is it compiled into your binary? Operated by your team? Consumed over the network
from a vendor?

These are orthogonal. AutoMapper and `.NET 8 runtime` are both "code-related"
— but compiled vs required-at-runtime are very different relationships. Azure
Event Hub and your team's Redis are both "network services" — but consumed-from-vendor
vs operated-by-team are very different relationships.

Two fields express both questions cleanly:

| Field | Question | Values |
|---|---|---|
| `dep-type` | What kind of thing? | library, service, api, tool, runtime, hardware, data |
| `dep-relationship` | How do you depend on it? | compiled, bundled, operated, consumed, invoked, required, physical |

---

## `dep-type` — Kind of Dependency

| Value | Meaning | Examples |
|---|---|---|
| `library` | Code package consumed via build system | AutoMapper, openraft, lodash, axios |
| `service` | Long-running process you can address | PostgreSQL, Redis, RabbitMQ |
| `api` | Remote interface you call over the network | Stripe API, Slack webhook, Anthropic Messages API |
| `tool` | CLI or executable invoked at build/deploy/runtime | Docker, kubectl, dotnet CLI, terraform |
| `runtime` | Execution environment that must be present | .NET 8, Node.js 20, JVM 21, Python 3.12 |
| `hardware` | Physical device | RPi 5, NVMe SSD, GPIO sensor |
| `data` | Static or external dataset | GeoIP database, ICU CLDR data, ML model file |

This is the **kind** of thing. It does not say how you use it.

---

## `dep-relationship` — How You Depend

| Value | Meaning | Failure mode |
|---|---|---|
| `compiled` | Linked into your binary at build time | Build breaks |
| `bundled` | Shipped with your deployment (sidecar, embedded binary, container image) | Missing at deploy time |
| `operated` | Network service your team runs and is responsible for | Your operational problem |
| `consumed` | Network service someone else runs (vendor, third party) | Their availability — beyond your control |
| `invoked` | Tool you call out to at build/deploy/run time | Missing in environment |
| `required` | Runtime environment that must be present | Wrong version available |
| `physical` | Physical hardware that must be present and healthy | Hardware failure |

This is **how** you depend on it. It does not say what kind of thing it is.

---

## The Combinations

Most real dependencies fit a natural type-relationship pairing:

| dep-type | dep-relationship | Example |
|---|---|---|
| library | compiled | AutoMapper NuGet, openraft Cargo crate, lodash npm |
| library | bundled | Init container with embedded gRPC client |
| service | operated | Your team's PostgreSQL, your team's Redis |
| service | consumed | Azure Cache for Redis, Supabase, Confluent Kafka |
| api | consumed | Stripe API, Slack webhook, Anthropic Messages API |
| tool | invoked | Docker, kubectl, dotnet CLI, terraform |
| runtime | required | .NET 8, Node.js 20, JVM 21 |
| hardware | physical | RPi 5, NVMe SSD |
| data | bundled | GeoIP DB shipped in the container |
| data | consumed | Real-time exchange rate feed |

Some pairings are unusual but valid (a `library` that is `bundled` rather than
`compiled` — for example, an embedded gRPC client in a sidecar). The schema
permits any combination.

---

## Front-Matter Schema

```yaml
---
id: DEP-007
title: AutoMapper
dep-type: library
dep-relationship: compiled
version: "12.0.1"
status: active                    # active | deprecated | evaluating | migrating
adrs: [ADR-019]                   # governing ADR — required (E013 if absent)
features: [FT-005, FT-006]        # features that use this dependency
interface: ~                      # optional, see below
availability-check: ~             # optional, relationship-specific (see below)
breaking-change-risk: low         # low | medium | high
---
```

The `dep-type` and `dep-relationship` fields are both required. `dep-relationship`
is new; existing DEP artifacts default to a relationship inferred from their
`dep-type` during migration (see Migration below).

---

## Interface and Availability Check by Relationship

The `interface:` and `availability-check:` fields take different shapes depending
on the relationship.

### `compiled`

```yaml
dep-type: library
dep-relationship: compiled
interface: ~                      # not applicable
availability-check: ~             # not applicable — build success is the check
```

A compiled dependency is verified by the build succeeding. No interface contract
beyond the version constraint. No availability check beyond build success.

### `bundled`

```yaml
dep-type: library
dep-relationship: bundled
interface:
  embed-path: lib/embedded-client.so
availability-check: "test -f lib/embedded-client.so"
```

The check is "the bundled artifact is present where expected."

### `operated`

```yaml
dep-type: service
dep-relationship: operated
interface:
  protocol: tcp
  port: 5432
  auth: password
  connection-string-env: DATABASE_URL
availability-check: "pg_isready -h $DB_HOST"
```

Full operational interface — your team runs it, you need to know how to start
and check it. Connection details, auth model, port.

### `consumed`

```yaml
dep-type: api
dep-relationship: consumed
interface:
  endpoint: https://api.stripe.com
  auth: bearer-token
  auth-env: STRIPE_API_KEY
  rate-limit: "100 req/sec per key"
  retry-policy: "exponential backoff, max 3 retries on 5xx"
availability-check: "curl -fsS -H 'Authorization: Bearer $STRIPE_API_KEY' https://api.stripe.com/v1/health"
```

Includes vendor-specific concerns: rate limits, retry policy, SLA-relevant
behaviour. The availability check is a real round-trip — you don't operate it,
so you must verify it responds.

### `invoked`

```yaml
dep-type: tool
dep-relationship: invoked
interface:
  command: docker
  min-version: "24.0"
availability-check: "docker version --format '{{.Server.Version}}'"
```

Tools are checked for presence and version.

### `required`

```yaml
dep-type: runtime
dep-relationship: required
interface:
  command: dotnet
  min-version: "8.0"
availability-check: "dotnet --version"
```

Runtime version constraint. Build-time and runtime check.

### `physical`

```yaml
dep-type: hardware
dep-relationship: physical
interface:
  device-path: /dev/nvme0n1
  min-capacity: 256GB
availability-check: "test -b /dev/nvme0n1 && nvme list | grep -q /dev/nvme0n1"
```

Hardware presence and basic health.

---

## What Each Combination Implies for Tests and Verification

The dep-relationship signals what TCs around this dependency typically need to verify:

### `compiled`
- Build succeeds with this dependency at the declared version
- Lock file is consistent with declared version
- No version conflicts with sibling dependencies

### `bundled`
- Deploy artifact contains the dependency
- Embedded artifact starts correctly when the parent process starts
- Bundled version matches declared version

### `operated`
- Service starts via documented mechanism
- Service responds to health check
- Service operates within declared resource limits
- Backup and restore procedures (if applicable)
- Failover behaviour (if HA)

### `consumed`
- Authentication works with current credentials
- Round-trip succeeds for a representative call
- Schema/contract has not changed (contract test)
- Retry policy behaves correctly on transient failure
- Circuit breaker engages on sustained failure
- Rate limiting respected

### `invoked`
- Tool is present in build/CI environment
- Tool version meets minimum requirement
- Tool's expected command works (no breaking syntax change)

### `required`
- Runtime version present
- Application starts under that runtime
- Runtime-specific features used are supported in the declared minimum version

### `physical`
- Device is present
- Device passes basic health check
- Device meets minimum capacity / spec

---

## `product dep` Commands

Existing commands gain relationship-aware behaviour:

```bash
product dep list                                  # all dependencies
product dep list --type library                   # filter by kind
product dep list --relationship consumed          # filter by relationship
product dep list --type api --relationship consumed   # combined filter

product dep show DEP-007                          # full detail
product dep check DEP-007                         # run availability-check
product dep bom                                   # full bill of materials

# New: filter the BOM by relationship to answer real questions
product dep bom --relationship operated           # what does my team operate?
product dep bom --relationship consumed           # what vendor APIs do we depend on?
product dep bom --relationship physical           # what hardware do we need?
```

The relationship filter on `product dep bom` answers questions that the
original BOM couldn't answer cleanly. "What third-party services do we depend
on?" is `--relationship consumed`. "What does my SRE team operate?" is
`--relationship operated`. "What needs a hardware presence check?" is
`--relationship physical`.

---

## `product dep classify` — Migration Helper

A one-shot command that scans existing DEP artifacts and proposes a
`dep-relationship` for each based on heuristics:

```bash
product dep classify

  Examining 23 dependencies...

  DEP-001  library  → compiled       (default for library type)
  DEP-002  service  → operated       (no auth-env, has port → likely team-operated)
  DEP-003  api      → consumed       (default for api type)
  DEP-004  service  → consumed       (auth-env: STRIPE_*, endpoint: stripe.com → vendor)
  ...

  Apply via: product dep classify --apply
  Or generate a change request: product dep classify --as-request > classify.yaml
```

The heuristics are not authoritative — the developer reviews and corrects.
Classification produces a request YAML by default rather than auto-applying,
so the human can adjust before the changes hit the graph.

---

## Validation

### E020 — Missing dep-relationship

```
error[E020]: DEP-007 has dep-type but no dep-relationship
  DEP-007: AutoMapper

  dep-relationship is required. Most likely value for dep-type=library: compiled

  Add via: product request change
    target: DEP-007
    op: set, field: dep-relationship, value: compiled
```

E020 fires during `product graph check` and `product request validate`.

### W031 — Unusual type-relationship pairing

```
warning[W031]: unusual dep-type and dep-relationship pairing
  DEP-007: AutoMapper
  dep-type: library
  dep-relationship: physical

  This pairing is permitted but uncommon. Verify it is intentional.
  Common pairings: library/compiled, library/bundled
```

W031 surfaces likely mistakes without blocking. The schema permits any
combination — sometimes the unusual pairing is correct.

### W032 — Availability check inappropriate for relationship

```
warning[W032]: availability-check on a 'compiled' dependency
  DEP-007: AutoMapper
  dep-relationship: compiled
  availability-check: "..."

  Compiled dependencies are verified by the build, not by an availability
  check. Consider removing the availability-check or changing the
  relationship if the dependency is actually bundled or invoked.
```

---

## Migration

Existing DEP artifacts have `dep-type` but no `dep-relationship`. The migration
path:

1. `product graph check` emits E020 for every existing DEP
2. `product dep classify` generates a change request inferring relationships
3. Developer reviews and edits the classification request
4. `product request apply` updates all DEPs at once

Inference defaults during classification:

| Existing dep-type | Inferred dep-relationship |
|---|---|
| library | compiled |
| service | operated (if no auth-env or vendor URL); consumed (if vendor URL) |
| api | consumed |
| tool | invoked |
| runtime | required |
| hardware | physical |

The `service` case is the only ambiguous one. Heuristics: presence of
auth-env hints at consumed; vendor URL patterns (stripe.com, *.azure.com,
*.amazonaws.com) indicate consumed; absence of these signals operated.

---

## `product.toml` Configuration

No new top-level config — the dependency type model is purely a front-matter
extension. Optional vendor URL patterns can be configured for the `dep classify`
command:

```toml
[dep-classify]
# URL patterns that indicate a 'consumed' (vendor) service
vendor-url-patterns = [
  "*.amazonaws.com",
  "*.azure.com",
  "*.googleapis.com",
  "stripe.com",
  "anthropic.com",
  "openai.com",
]
```

---

## Session Tests

```
ST-360  dep-relationship-field-parses
ST-361  dep-relationship-invalid-value-emits-e006
ST-362  dep-missing-relationship-emits-e020
ST-363  dep-classify-infers-from-existing-fields
ST-364  dep-classify-generates-request-yaml
ST-365  dep-bom-filter-by-relationship
ST-366  dep-list-filter-by-relationship
ST-367  unusual-pairing-emits-w031
ST-368  availability-check-on-compiled-emits-w032
ST-369  dep-classify-vendor-url-pattern-detection
ST-370  request-create-dep-requires-relationship
```

---

## Invariants

- `dep-relationship` is required on all DEP artifacts after migration.
  E020 fires until every dependency has one.
- `dep-type` and `dep-relationship` are independently validated. The schema
  permits any combination; W031 flags unusual pairings as warnings.
- `product dep bom --relationship X` answers questions about the dependency
  landscape that the original `dep-type`-only BOM could not.
- The `availability-check` field's interpretation depends on `dep-relationship`.
  Compiled dependencies need none. Consumed dependencies need a real round-trip.
  This is documented but not strictly enforced.
- `product dep classify` never auto-applies. It always produces a request YAML
  for human review. The developer is the authority on the team's relationship
  to each dependency.
