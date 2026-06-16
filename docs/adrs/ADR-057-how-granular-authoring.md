---
id: ADR-057
title: How elements are authored granularly via add/set on the contract file
status: accepted
features:
- FT-115
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:578250e7309910918c37ff049e60d1cec7b0055b22206ccd95dc33169b6d2083
source-files:
- product-core/src/pf/how_edit.rs
- product-cli/src/commands/how.rs
- product-cli/src/commands/how_fields.rs
---

## Context

FT-111 made the How contract loadable, validatable, and projectable, but
authorable only by hand-editing the YAML or scaffolding it whole. The What
graph, by contrast, has element-level CRUD (`product domain new …`). Building
an archetype's How — the Why cascade (top decisions → principles → patterns)
then the application and infrastructure contracts — wants the same granular,
incremental authoring from the CLI.

## Decision

Add granular mutation of the How contract: `product how add <element> <id>`
and `product how set <contract> --id …`, backed by typed functions in
`pf::how_edit`.

- **Add** appends a `decision`, `principle`, `pattern`, `interface`,
  `app-statement`, or `resource`.
- **Set** replaces the singleton `app-contract` or `infra-contract`.
- The CLI auto-initialises an empty How (keyed to the repo product) on first
  write, so the contract is built up element by element from nothing.

Two coherence rules live in `pf::how_edit`:

1. **Cross-cascade id uniqueness** — decisions, principles, patterns, and
   interfaces share one id namespace (they reference each other by id), so a
   duplicate id anywhere in the cascade is rejected. Nested ids
   (app-statement, resource) are unique within their contract.
2. **Parent-before-child** — `add app-statement` requires the application
   contract to be set; `add resource` requires the infrastructure contract.
   `set` preserves already-added statements/resources, so re-setting a
   contract's metadata never silently drops them.

Each mutation persists the file; conformance is checked on demand with
`product how validate`, since a How under construction is expected to be
non-conformant (e.g. an applied principle not yet enforced) until complete.

## Rationale

- Granular authoring matches the What graph's CRUD ergonomics and lets the
  Why cascade and contracts be built in the natural order without hand-editing
  YAML.
- A shared id namespace for the cascade is what makes `licenses`/`realizes`/
  `realized_by` references resolvable; enforcing uniqueness up front prevents
  ambiguous cross-references.
- Persisting without hard-failing on incomplete conformance keeps the build
  incremental; `validate` is the explicit gate.

## Rejected alternatives

- **Hand-edit the YAML only.** Rejected: inconsistent with the What graph's
  CRUD and error-prone for the cross-referencing cascade.
- **Validate-and-reject on every mutation.** Rejected: a How under
  construction is legitimately non-conformant between steps; blocking each
  add would make incremental authoring impossible.
- **One `add-decision`/`add-principle`/… verb each.** Rejected in favour of a
  single `add <element>` (and `set <contract>`), mirroring `domain new <kind>`
  and keeping the subcommand surface small.

## Test coverage

- TC-950 — build a full conformant How from scratch via add/set.
- TC-951 — a duplicate id across the Why cascade is rejected.
- TC-952 — add resource requires the infrastructure contract.
- TC-953 — an unknown element kind is rejected.
- TC-954 — re-setting a contract preserves already-added statements.
