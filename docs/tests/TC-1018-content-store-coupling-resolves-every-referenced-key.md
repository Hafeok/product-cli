---
id: TC-1018
title: content store coupling resolves every referenced key
type: scenario
status: passing
validates:
  features:
  - FT-142
  adrs:
  - ADR-085
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1018_content_store_coupling_resolves_every_referenced_key
last-run: 2026-06-22T18:49:08.027555139+00:00
last-run-duration: 0.5s
---

## Scenario — content coverage over (key, locale) is the content analogue of reification coverage

**Given** a content-store manifest that resolves every (content key, locale) the
application's UI steps and shell reference, and a captured What graph carrying
those content references,
**When** the user runs the coupling check,
**Then** the process exits 0 and the check reports content coverage complete — no
referenced key is left unresolved in any claimed locale.

**And given** a manifest that cannot resolve one referenced (key, locale) pair
(e.g. `cart.empty.message` in `de`), **when** the user runs the coupling check,
**then** the process exits non-zero and the check declares the store
**non-conforming for that locale**, naming the missing (key, locale) pair — the
content analogue of reification coverage over (AIO, context).

## Validates

- FT-142 — Content Store Conformance Profile (preview)
- ADR-085 — Preview conformance profiles for the design system and the content store