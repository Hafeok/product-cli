---
id: FT-121
title: product decider — derive an aggregate's executable signature and validate drift
phase: 6
status: complete
depends-on:
- FT-110
adrs:
- ADR-061
tests:
- TC-946
- TC-947
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `decider` subcommand family; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Derivation + validation live in the pure `pf::decider` slice; the CLI is a thin BoxResult adapter.
  ADR-048: Reads the captured What graph; writes only the derived decider file on `derive`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd; `pf::decider`/`pf::rules_decider` carry unit tests. No property or session dimension for a derivation.
  ADR-040: The Decider is a What-side artifact at the What/How boundary; its conformance rules compose the existing event-model shapes; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

§3.3 of the framework defines the **Decider** — the executable form of an
aggregate's behaviour. Its signature is not authored but *derived from* the
event model: it handles exactly the commands targeting its aggregate, emits only
events those commands sanction, evolves from the events that change it, and
rejects via its invariants. `product decider` derives that signature from the
captured What graph and validates an authored Decider against it.

## Functional Specification

### Inputs

- The captured What graph for a product (`--product` to override the default).
- An aggregate entity id (`derive`) or a decider id (`validate`/`show`).

### Behaviour

- `product decider derive <aggregate>` — derive the full signature from the What
  graph (`handles` = commands targeting the aggregate, `emits` = the union of
  their emitted events, `evolves_from` = events that change it, `rejects` = its
  invariants) and write it to `.product/deciders/<aggregate>-decider.yaml`.
  Refuses to overwrite without `--force`.
- `product decider validate <name>` — validate an authored Decider against the
  event model via the three §3.3 drift rules, run as graph rules over the
  combined What + Decider projection:
  - **No foreign commands** — every handled command targets the aggregate.
  - **Command coverage** — every command targeting the aggregate is handled.
  - **Output-alphabet containment** — every emitted event is sanctioned by a
    handled command.
  Plus a structural check that the aggregate it decides for is a real entity.
  Exits 1 on any violation, listing each.
- `product decider show <name>` / `list` — the signature / the deciders present.

### Error handling

- Deriving for an id that is not an entity in the What graph is a clear error.
- Validating with no captured What graph points the user at `product author
  domain`.

## Out of scope

- The Decider's **decision logic** (the `decide`/`evolve` function bodies) and
  the **before-realisation simulation** against flow-derived scenarios (§3.3)
  are a separate increment; this feature establishes the conformant signature
  and its anti-drift rules.

## Acceptance

- TC-946 — derive a signature and validate it conformant; show / list work.
- TC-947 — a Decider handling a foreign command is non-conformant.
