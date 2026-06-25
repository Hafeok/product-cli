---
id: ADR-090
title: System identity is What; deployment identity is How
status: accepted
features:
- FT-148
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:**

Framework §3.2.5 makes the system a first-class What node. Until now the page
graph had a single, flat `ApplicationRoot` with no node naming the system it
belongs to, and nothing distinguished *what the product is and reaches* (its
name, kind, purpose, target platforms/classes) from *where it is deployed* (its
production domain name, App Store bundle id, chosen runtime). A What may also
describe several systems over one shared domain, which a single flat root cannot
express.

**Decision:**

The `System` node carries only What-side identity and reach: `kind`, `purpose`,
`target_platforms`, `target_classes`, and a `rootsAt` edge to the
`ApplicationRoot` its page graph roots at. A flow declares the one system it
belongs to via a `system` ownership edge (`pf:systemOf`). Deployment identity —
domain name, bundle ids, runtimes — is explicitly **not** carried by the system
node; it lives in the infrastructure/runtime contract (§4.2), because it varies
per deployment and is frozen once chosen. The `ApplicationRoot` stays a separate
node (rather than being folded into the system) so the page-graph machinery and
"top-level is derived" rule are unchanged, and a system simply points at its root.

**Rationale:**

Keeping meaning-and-reach above the What/How line and concrete-address below it
preserves the central split (§2): product and design own the system's identity,
engineering owns its deployment. Modelling ownership as a flow→system edge keeps
the domain model shared while letting each system be a distinct surface, exactly
the multi-system story §3.2.5 requires. Reusing the existing `ApplicationRoot`
avoids duplicating navigation state in two places.

**Rejected alternatives:**

- **Fold the root into the system node.** Rejected: it would duplicate the
  page-graph root machinery, break the derived "top-level" rule (§3.2.4), and
  couple system identity to navigation structure.
- **Put deployment identity (domain name, bundle id) on the system.** Rejected:
  those are realisation that varies per deployment; placing them above the line
  violates §3.2.5 and the §4.2 runtime-contract boundary.
- **Enforce that every flow names a system immediately.** Rejected for now: it
  would invalidate existing What graphs that predate the system node; ownership
  is validated when declared and the completeness rule is deferred.

**Test coverage:** TC-1030, TC-1031.
