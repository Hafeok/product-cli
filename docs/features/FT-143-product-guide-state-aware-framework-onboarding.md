---
id: FT-143
title: product guide — state-aware framework-graph onboarding
phase: 8
status: complete
depends-on:
- FT-125
adrs:
- ADR-088
tests:
- TC-1019
domains:
- api
- data-model
domains-acknowledged:
  ADR-040: A read-only derived view at the What→How→Delivery boundary; it composes existing state probes and does not touch the verify pipeline.
  ADR-018: A scenario TC drives the binary through assert_cmd; `product_core::guide` carries unit tests over every stage. No property or session dimension for a read-only derivation.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-048: Reads the captured framework graph under `.product/`; writes nothing.
  ADR-051: The TC declares `observes:` (stdout, exit-code) and asserts on those surfaces.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-041: Additive — a new read-only `guide` command + `product_guide` MCP tool; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-050: PAT-001 (slice + adapter) and PAT-002 (MCP write parity) are cited via `patterns:`; no new implementation pattern is introduced.
  ADR-043: The guidance logic lives in the pure `product_core::guide` slice (probe + stage decision + render); the CLI and the MCP handler are thin adapters.
  ADR-049: Not a context-bundle/template command; no template surface changes.
patterns:
- PAT-001
- PAT-002
---

## Description

The framework graph (What/How/Delivery) had every lifecycle command but no
on-ramp — each command stood alone and nothing connected them. `product guide`
is the spine: it probes the framework-graph state and prints **where the user is
in the journey**, what the current step means, and the **exact next command** —
papering over the authoring papercuts (relations required up front,
`slice --anchor`). It is exposed identically as the `product_guide` MCP tool so
an agent can orient a user (CLI↔MCP parity, FT-118).

## Functional Specification

### Inputs

- The captured framework graph under `.product/` for the default product (the
  domain session, `how-contract.yaml`, and the deciders/slices/deliverables/
  releases directories). No arguments.

### Behaviour

- Probe a `FrameworkState` (a pure snapshot: What node counts + conformance, an
  example command id, How presence, slice/deliverable counts, an example slice
  id) via the shared `product_core::guide::FrameworkState::probe`.
- Compute the current **stage** as the first unmet step in the strict order
  CaptureWhat → FixWhat → AuthorHow → CarveSlice → WrapDeliverable → BuildIt.
- Render: a journey **checklist**, the stage **headline**, a one-line **concept**
  reminder, and the concrete **next command(s)** — naming a real command as a
  `slice --anchor` and a real slice as a `deliverable --slice` when known,
  placeholders otherwise.
- `--format json` emits the structured guidance (stage, next_steps, progress).
- `product_guide` MCP tool returns the same guidance plus a rendered `text`
  field, backed by the same shared module.

### Error handling

- A fresh/empty graph yields the CaptureWhat stage (no error) — every missing
  piece reads as zero.

## Out of scope

- The lifecycle commands it points at (`author domain`, `domain`, `how`,
  `slice`, `deliverable`, `decider`, `build`) are pre-existing features.
- Authoring content (the guide is strictly read-only).

## Acceptance

- TC-1019 — on a fresh repo, `product guide` reports the CaptureWhat stage with
  an unticked checklist and suggests `product author domain`; `--format json`
  carries the structured stage. (ADR-088)
