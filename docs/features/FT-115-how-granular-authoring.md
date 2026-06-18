---
id: FT-115
title: product how add/set — granular authoring of the Why cascade and contracts
phase: 6
status: complete
depends-on:
- FT-111
adrs:
- ADR-057
tests:
- TC-950
- TC-951
- TC-952
- TC-953
- TC-954
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — new `how add`/`how set` verbs; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Followed — mutation logic lives in the pure `pf::how_edit` slice; the CLI is a thin BoxResult adapter with a flags struct.
  ADR-048: Reads/writes a How-contract YAML (default `.product/how-contract.yaml`); the FT/ADR/TC graph is untouched.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Five scenario TCs drive the binary through the assert_cmd harness; the pf::how_edit slice carries unit tests. No property or session dimension for a file mutator.
  ADR-040: The How is a structural artifact; mutations are typed and conformance is checked by `how validate`; the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product how add` and `product how set` build an archetype's How contract
element by element — the Why cascade (top decisions → principles → patterns),
then the application and infrastructure contracts — without hand-editing YAML.
This gives the How the same granular authoring ergonomics the What graph has
with `product domain new`.

## Functional Specification

### Inputs

- `product how add <element> <id> [--fields…]` where `<element>` is one of
  `decision`, `principle`, `pattern`, `interface`, `app-statement`,
  `resource`.
- `product how set <contract> --id <id> [--fields…]` where `<contract>` is
  `app-contract` or `infra-contract`.
- A `--file` (default `.product/how-contract.yaml`), auto-initialised on first
  write keyed to the repo product name.

### Behaviour

- `add decision` — `--decision`, `--rationale`, `--applies-when`,
  `--does-not-apply-when`, `--licenses`, `--enforced-by`.
- `add principle` — `--statement`, `--licensed-by`, `--realized-by`,
  `--enforced-by`.
- `add pattern` — `--shape`, `--realizes`, `--applied-by`, `--enforced-by`.
- `add interface` — `--surface`, `--standard`, `--derived-from`.
- `set app-contract` — `--language`, `--runtime`, `--layering`,
  `--feature-organization`, `--persistence-model`, `--cross-cutting`;
  `add app-statement <id> --statement [--enforced-by]`.
- `set infra-contract` — `--satisfies` (frozen at Discovery);
  `add resource <id> --kind --choice [--satisfies-statement] [--depends-on]`.
- Each mutation persists the contract. Run `product how validate` to check
  conformance (incl. the crown trace-truth rule) when the How is complete.

### Error handling

- A duplicate id across the Why cascade (decisions/principles/patterns/
  interfaces) is rejected.
- `add app-statement` / `add resource` require the parent contract to be set
  first; `set` preserves already-added statements/resources.
- An unknown element or target is rejected with the accepted values.

## Out of scope

- It does not delete or rename How elements (edit the YAML for those); this
  covers additive build-up and contract (re)setting.
- It does not validate on every mutation — a How under construction is
  legitimately incomplete; `how validate` is the gate.

## Acceptance

- TC-950 — build a full conformant How from scratch via add/set.
- TC-951 — a duplicate id across the Why cascade is rejected.
- TC-952 — add resource requires the infrastructure contract.
- TC-953 — an unknown element kind is rejected.
- TC-954 — re-setting a contract preserves already-added statements.
