---
id: ADR-072
title: Workers are capabilities resolved by role with escalation; runners are claude or litellm
status: accepted
features:
- FT-131
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
source-files:
- product-core/src/pf/capability.rs
- product-cli/src/commands/worker.rs
- product-cli/src/commands/build.rs
---

## Context

`build` hardcoded a single `claude -p` spawn — the SPMC "Model" slot was fixed.
The sibling project decision-cli has a mature, capability-routed worker model
(`dec:Capability` catalog, `dec:RoleBinding` with escalation ladders, a LiteLLM
proxy whose `model_name`s *are* the capability tags, and per-role Python
workers). The model is portable; the runtime (Python workers + ModelRouter) is
not directly linkable from Rust, but the LiteLLM proxy is a language-agnostic
HTTP seam.

## Decision

Adopt the capability + role-binding model natively, and abstract the runner:

- **`pf::capability`** (pure): `Capability { id, endpoint, model_identifier,
  tier, status }`, `RoleBinding { role_id, default_capability, escalation_steps,
  active }`, and `Catalog::resolve(role, triggers)` — start at the default, walk
  the ladder, and the highest rung whose triggers fire wins. Mirrors
  decision-cli's `dec:Capability`/`dec:RoleBinding`, so its `capabilities.yaml`
  and `role-bindings.yaml` load directly.
- **Runners** (`commands/worker`): `endpoint: claude` → a `claude -p`
  subprocess (agentic, tool-using); `endpoint: litellm` → an HTTP POST to the
  LiteLLM proxy with `model = capability id` — reusing decision-cli's proxy to
  reach Scaleway/Anthropic/any provider with no provider code here.
- **`build`** resolves `--role` to a capability and dispatches via the matching
  runner; with no catalog it falls back to a built-in claude capability so
  existing behaviour is unchanged.

## Rationale

- The capability/role model is the principled SPMC "Model" layer: model choice
  becomes catalog-driven and escalating, not hardcoded — and their role ≈ our
  cell/task-type, so it slots cleanly into the framework.
- The LiteLLM proxy is the one cross-language integration point that needs no
  shared code: capability tags are litellm `model_name`s, so an HTTP call routes
  to any provider. That is "workers that aren't just claude" with the least
  coupling.
- Keeping the agentic path on the `claude` subprocess preserves tool use for
  code work; one-shot roles (authoring, quality, classification) go via litellm.

## Rejected alternatives

- **Link decision-cli's Python ModelRouter / workers.** Rejected for now:
  cross-language, and the LiteLLM proxy already exposes every model over HTTP. A
  later increment may invoke their Python workers over the pipeline-worker-sdk
  wire protocol where their specific logic is wanted.
- **A bespoke provider client per endpoint in Rust.** Rejected: re-implements
  what the LiteLLM proxy already does (routing, fallbacks, telemetry, keys).

## Test coverage

- TC-985 — `worker resolve` default + escalation by trigger.
- TC-986 — `build` resolves the worker by role.
- `pf::capability` units: default vs escalated resolution, unknown role, dangling
  capability, unknown trigger.
