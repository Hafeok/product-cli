---
id: FT-131
title: worker capability catalog — role to capability with escalation, claude or litellm
phase: 6
status: complete
depends-on:
- FT-130
adrs:
- ADR-072
tests:
- TC-985
- TC-986
- TC-989
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `worker` family + a runner abstraction in `build`; nothing removed.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: The capability/role model + resolution live in pure `pf::capability`; the CLI owns catalog I/O + the runners.
  ADR-048: Reads `.product/{capabilities,role-bindings}.yaml`; `init` writes the seeds; dispatch is live (claude subprocess / litellm HTTP).
  ADR-051: Every TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary via assert_cmd; `pf::capability` carries unit tests over resolution + validation.
  ADR-040: The catalog is the SPMC Model layer; it composes `build`, not the verify pipeline.
patterns:
- PAT-001
---

## Description

Ported from decision-cli's worker model: `build` no longer hardcodes `claude`.
A **capability** is a catalogued model (its `endpoint` picks the runner, `id` is
the routing tag); a **role binding** maps a role (e.g. `implementer`) to a
default capability plus an **escalation ladder** (triggers bump it up the tiers).
`build` resolves the work's role to a capability and dispatches to the matching
runner — a `claude` subprocess (agentic) or the `litellm` proxy (any provider
behind a capability tag).

## Functional Specification

### Catalog (`.product/capabilities.yaml` + `role-bindings.yaml`)

- `Capability { id, endpoint(claude|litellm), model_identifier, tier, status }`.
- `RoleBinding { role_id, default_capability, escalation_steps[], active }`,
  each step `{ capability, triggers[] }` from a fixed trigger vocabulary
  (`audit_fail`, `confidence_below_0.7`, `prior_attempts_ge_5`,
  `stakes_foundational`, …).

### Commands

- `product worker init` — scaffold seed catalogs (claude + litellm capabilities,
  `implementer`/`verifier` bindings).
- `product worker list` — capabilities + role bindings (with ladders).
- `product worker resolve <role> [--trigger …]` — the resolved capability;
  firing triggers escalate up the ladder.
- `product worker check` — validate the catalog (bindings resolve; triggers known).
- MCP parity: `product_worker_list`, `product_worker_resolve`.

### Dispatch in `build`

`product build <deliverable> [--role implementer]` resolves the role to a
capability and dispatches via the runner: `claude -p` for `endpoint: claude`, or
an HTTP POST to the LiteLLM proxy (`LITELLM_BASE_URL`/`LITELLM_API_KEY`,
`model = capability id`) for `endpoint: litellm`. Without a catalog, `build`
falls back to the built-in claude capability (unchanged behaviour).

### Proxy routing

`litellm` capabilities — and the `scaleway`/`anthropic` aliases — all route
through the **LiteLLM proxy** at `LITELLM_BASE_URL`, sending the capability `id`
as the proxy `model_name`. The proxy holds the provider keys and maps that tag to
a provider model, so **Scaleway is reached via a proxy model group, not a direct
API call** here. `worker check` validates that every capability's `endpoint` is
one of the known runners (`claude`, `litellm`, `worker`, `scaleway`, `anthropic`)
— catching the misconfiguration of pointing `LITELLM_BASE_URL` at a provider's
own API instead of the proxy.

### Behaviour

`build` resolves the work's role to a capability through the catalog's escalation
ladder, then dispatches by `endpoint` to the matching runner. `worker
{init,list,resolve,check,run}` scaffold and inspect the catalog; resolution
applies the active triggers, climbing rungs weakest-first.

### Error handling

- An unknown role or empty catalog falls back to the built-in default capability
  rather than failing the build.
- A capability naming an unknown endpoint, or a binding/step referencing a
  missing capability, is reported by `worker check` (exit 1). A model or
  transport failure during dispatch is retried, then surfaced without corrupting
  state.

## Out of scope

- Wiring escalation triggers from live signals (confidence, prior attempts) into
  `build` — the resolution honours triggers; feeding them automatically is next.
- Our own first-party worker (a future increment).

## Acceptance

- TC-985 — `worker resolve` returns the default capability, and escalates when a
  trigger fires.
- TC-986 — `build` resolves the worker by role and reports it.
