# Product Documentation Artifact Specification

> Standalone reference for documentation artifacts — first-class graph nodes
> linking specifications to in-repo documentation files.
>
> New artifact type: `DOC-XXX`
> New feature field: `docs`
> New product.toml section: `[documentation]`
> New commands: `product doc list/show/stale/bundle/check`
> New codes: E018, W025, W026, W027

---

## Overview

Documentation is a first-class artifact in the knowledge graph. A `DOC-XXX`
artifact is a graph node that links one or more features and ADRs to a documentation
file that lives in the repository. Documentation files are committed, diffable,
reviewed in PRs, and version-controlled alongside the code they describe.

Three properties distinguish documentation artifacts from other artifact types:

1. **The file must exist in the repository.** `location` is always a path relative
   to the repo root. E018 fires if the file doesn't exist — same severity as a
   broken link.

2. **Staleness is detected via git.** When a linked feature gets a new completion
   tag and the documentation file hasn't been touched since the previous tag,
   W025 fires. No LLM required — this is a structural git query.

3. **Content is the user's responsibility.** Product validates that the file
   exists, is non-empty, and is current relative to its linked features. It never
   parses or validates the content. `product doc bundle` produces the doc content
   plus linked spec context for an LLM to review — the LLM call is the user's
   concern.

---

## Front-Matter Schema

```yaml
---
id: DOC-001
title: PiCloud CLI Reference
type: reference          # reference | conceptual | operational
status: current          # draft | current | stale | deprecated
format: markdown         # markdown | openapi | asyncapi | adoc | custom
location: docs/reference/cli.md    # required — repo-relative path, must exist
features: [FT-001, FT-002, FT-003] # features this document covers
adrs: [ADR-001, ADR-007]           # decisions that shaped this document
---

CLI reference for the PiCloud binary. Covers all subcommands,
flags, environment variables, and exit codes.
```

### Fields

| Field | Required | Description |
|---|---|---|
| `id` | yes | `DOC-XXX` — assigned by Product, sequential |
| `title` | yes | Human-readable document name — immutable once set |
| `type` | yes | Document category — see Types below |
| `status` | yes | `draft` / `current` / `stale` / `deprecated` |
| `format` | yes | File format — informational, not validated by Product |
| `location` | yes | Repo-relative path to the documentation file |
| `features` | no | Features this document covers |
| `adrs` | no | ADRs that shaped this document's content |

At least one of `features` or `adrs` must be non-empty — an orphaned DOC artifact
is W001 (same as an orphaned ADR).

---

## Documentation Types

### `reference`

Describes what a thing is and how to use it. API reference, CLI reference,
configuration reference, schema reference. Typically precise and complete.
Changes with every API change.

**Examples:**
- `docs/reference/cli.md` — all commands and flags
- `docs/reference/api.yaml` — OpenAPI specification
- `docs/reference/config.md` — `product.toml` full reference
- `docs/reference/picloud-resource-types.md` — all `.picloud` resource types

### `conceptual`

Explains why things work the way they do. Architecture guides, design principles,
tutorials, how-to guides. Consumed by developers integrating or extending the system.
Changes less frequently than reference docs.

**Examples:**
- `docs/guides/event-sourcing.md` — how PiCloud's event log works
- `docs/guides/raft-consensus.md` — consensus model explanation
- `docs/guides/deploying-products.md` — how to write and deploy a product

### `operational`

How to run, monitor, and troubleshoot. Runbooks, deployment guides, incident
response, observability setup. Changes with operational patterns.

**Examples:**
- `docs/ops/bootstrap.md` — cluster bootstrap procedure
- `docs/ops/runbook-leader-failover.md` — what to do when leader election fails
- `docs/ops/observability.md` — metrics, traces, and alerts reference

---

## Feature Front-Matter Extension

Features gain a `docs` field:

```yaml
---
id: FT-001
title: Cluster Foundation
phase: 1
status: complete
adrs: [ADR-001, ADR-002]
tests: [TC-001, TC-002]
docs: [DOC-001, DOC-003]     # new field — documentation artifacts covering this feature
uses: [DEP-001]
---
```

The link is bidirectional — DOC-001's `features` list and FT-001's `docs` list
both declare the relationship. Request model enforces consistency.

---

## Git-Based Staleness Detection

Staleness is detected the same way as implementation drift (ADR-023) — using
git completion tags as anchors.

### When W025 fires

W025 fires when all of these are true:
- A feature linked to the DOC artifact has a completion tag (`product/FT-XXX/complete`)
- The documentation file (`location`) has NOT been modified since the feature's
  most recent completion tag
- The feature has been re-verified at least once since the documentation file
  was last modified

```bash
# Structural query Product runs internally
TAG=$(git describe --match "product/FT-001/complete*" --abbrev=0 2>/dev/null)
git log "$TAG"..HEAD -- docs/reference/cli.md
# If this returns nothing → documentation not touched since last feature completion → W025
```

W025 is a warning, not an error. It says: "FT-001 was re-verified after
docs/reference/cli.md was last modified — the documentation may not reflect
the current implementation." The developer decides whether the change warranted
a documentation update.

### When W027 fires

W027 fires when the reverse is true: the documentation file was modified but the
linked feature has NOT been re-verified since the modification.

```bash
# The doc changed more recently than the last completion tag
git log --format="%H" "$TAG"..HEAD -- docs/reference/cli.md
# If this returns commits → doc changed after completion → W027
```

W027 says: "docs/reference/cli.md was modified after FT-001's last completion
tag — verify that the documentation change is aligned with the current
implementation by running `product verify FT-001`."

### Neither fires when

- The doc file was modified after the feature's completion tag AND the feature
  was subsequently re-verified (both are in sync)
- The feature has never been verified (no completion tag exists)

### Manual status override

If the developer knows W025 is a false positive — the feature change was cosmetic
and the documentation is still accurate — they apply a change request:

```yaml
type: change
reason: "Documentation reviewed — no update needed after FT-001 performance fix"
changes:
  - target: DOC-001
    mutations:
      - op: set
        field: status
        value: current
```

This sets `status: current` explicitly and suppresses W025 until the next
completion tag is created.

---

## Validation Codes

| Code | Tier | Condition |
|---|---|---|
| E018 | Integrity | DOC artifact `location` path does not exist in the repository |
| W025 | Documentation | Doc file not modified since linked feature's last completion tag |
| W026 | Documentation | Complete feature has no linked DOC artifact |
| W027 | Documentation | Doc file modified after linked feature's last completion tag but feature not re-verified |

### E018 — Missing documentation file

```
error[E018]: documentation file not found
  DOC-001: PiCloud CLI Reference
  location: docs/reference/cli.md
  File does not exist in the repository.

  Create the file or update the location field:
    product request change:
      target: DOC-001
      op: set, field: location, value: docs/reference/cli-reference.md
```

### W025 — Potentially stale documentation

```
warning[W025]: documentation may be stale
  DOC-001: PiCloud CLI Reference
  location: docs/reference/cli.md

  Linked feature FT-001 was re-verified on 2026-04-14
  (tag: product/FT-001/complete-v2)

  docs/reference/cli.md was last modified on 2026-04-11
  (before the re-verification)

  Review the documentation and update if needed, then:
    product request change:
      target: DOC-001
      op: set, field: status, value: current
```

### W026 — Undocumented complete feature

```
warning[W026]: complete feature has no documentation
  FT-005: Rate Limiting
  status: complete (product/FT-005/complete 2026-04-14)

  No DOC artifact is linked to FT-005.
  Create documentation and link it via product request create.
```

W026 is advisory by default. Configurable as a completion blocker:

```toml
[documentation]
require-on-complete = false    # default — W026 advisory
                               # set true to block feature completion
```

### W027 — Documentation ahead of verification

```
warning[W027]: documentation modified but feature not re-verified
  DOC-001: PiCloud CLI Reference
  location: docs/reference/cli.md

  docs/reference/cli.md was modified on 2026-04-15
  Last verification of linked FT-001: 2026-04-14

  The documentation may have drifted from the verified implementation.
  Run: product verify FT-001
```

---

## Commands

```bash
# Discovery and inspection
product doc list                        # all DOC artifacts
product doc list --type reference       # filter by type
product doc list --status stale         # artifacts with stale status
product doc list --feature FT-001       # docs covering a feature
product doc show DOC-001                # full artifact detail
product doc stale                       # all docs triggering W025

# Checking
product doc check                       # run all doc validation rules
product doc check --changed             # only docs affected by recent commits
product doc check FT-001                # docs covering a specific feature

# LLM-ready output
product doc bundle DOC-001              # doc content + linked spec context → stdout
product doc bundle --stale              # bundles for all stale docs → stdout
```

### `product doc bundle`

Produces a structured markdown document for LLM review — the documentation
content plus the linked spec context, with a diff of changes since the last review:

```markdown
# Documentation Review Input: DOC-001 — PiCloud CLI Reference

## Instructions

Review the documentation against the current specification.
Identify: (1) documented behaviours that no longer match the spec,
(2) spec behaviours not covered by the documentation,
(3) outdated examples or command syntax.

Output: one JSON object per finding with fields:
  type: "stale" | "missing" | "incorrect"
  description: string
  location: line number or section in the documentation

## Current Documentation

[full content of docs/reference/cli.md]

## Linked Features — Current Specification

[context bundle for FT-001, FT-002, FT-003 at depth 1]

## Changes Since Last Review

[git diff of docs/reference/cli.md since product/FT-001/complete]
[git diff of docs/reference/cli.md since product/FT-002/complete]
[git diff of docs/reference/cli.md since product/FT-003/complete]
(deduplicated — earliest relevant tag used as baseline)
```

---

## `product.toml` Configuration

```toml
# Documentation settings
[documentation]
require-on-complete = false    # true → W026 becomes E-class, blocks completion
stale-check = true             # false → disable W025/W027 globally
```

---

## Repository Layout

Documentation files live anywhere in the repository. The DOC artifact's `location`
field is the authoritative pointer. Convention for a typical project:

```
/docs
  /reference
    cli.md
    config.md
    api.yaml
  /guides
    event-sourcing.md
    deploying-products.md
  /ops
    bootstrap.md
    runbook-leader-failover.md
  /product-specs       ← Product-managed artifacts
    /features
    /adrs
    /tests
    /deps
    /docs              ← DOC artifact files (the metadata)
      DOC-001-cli-reference.md
      DOC-002-event-sourcing-guide.md
```

The DOC artifact file is in `docs/product-specs/docs/`. It contains only front-matter
and a brief description. The actual documentation is in `docs/reference/cli.md`
or wherever the team keeps it. The `location` field is the link between them.

---

## Request Model Integration

### Create a documentation artifact

```yaml
type: create
reason: "Document the CLI reference for Phase 1 completion"
artifacts:
  - type: doc
    ref: doc-cli-ref
    title: PiCloud CLI Reference
    doc-type: reference
    format: markdown
    location: docs/reference/cli.md
    features: [FT-001, FT-002, FT-003]
    adrs: [ADR-001, ADR-007]

changes:
  - target: FT-001
    mutations:
      - op: append
        field: docs
        value: ref:doc-cli-ref
  - target: FT-002
    mutations:
      - op: append
        field: docs
        value: ref:doc-cli-ref
  - target: FT-003
    mutations:
      - op: append
        field: docs
        value: ref:doc-cli-ref
```

`product request validate` checks that `docs/reference/cli.md` exists before
accepting the request. E018 if it doesn't — create the file first, then apply
the request.

### Mark documentation as reviewed

```yaml
type: change
reason: "Reviewed DOC-001 after FT-003 performance improvement — no update needed"
changes:
  - target: DOC-001
    mutations:
      - op: set
        field: status
        value: current
```

### Deprecate documentation

```yaml
type: change
reason: "CLI reference superseded by generated API docs"
changes:
  - target: DOC-001
    mutations:
      - op: set
        field: status
        value: deprecated
```

---

## Integration with `product verify`

In the unified verify pipeline (product-verify-and-llm-boundary-spec.md), documentation
checks run as part of stage 2 (graph structure):

```
[2/6] Graph structure ............ ⚠  4 warnings
          W025  DOC-001 may be stale — FT-001 re-verified since last doc update
          W026  FT-005 complete with no documentation
          W012  FT-013 has no bundle measurement
          W016  FT-002 has 1 unimplemented TC
```

`product verify --level unit,component` skips W025/W026/W027 checks — these are
release-readiness signals, not fast-feedback signals.

### `product impact FT-003`

Impact analysis now includes documentation:

```
Impact analysis: FT-003 — RDF Store

Direct dependents:
  Features:       FT-007, FT-008 (depend-on)
  Tests:          TC-020, TC-021 (validate)
  Documentation:  DOC-001 (CLI Reference), DOC-004 (Architecture Guide)
                  ← these docs may need updating
```

---

## Complete Example: PiCloud Documentation Graph

```
DOC-001  PiCloud CLI Reference          reference  docs/reference/cli.md
  covers: FT-001, FT-002, FT-003, FT-004
  adrs:   ADR-001, ADR-007, ADR-008

DOC-002  Event Sourcing Architecture    conceptual  docs/guides/event-sourcing.md
  covers: FT-003, FT-005
  adrs:   ADR-004, ADR-005, ADR-008

DOC-003  Cluster Bootstrap Guide        operational  docs/ops/bootstrap.md
  covers: FT-001
  adrs:   ADR-002, ADR-003

DOC-004  Raft Consensus Explanation     conceptual  docs/guides/raft-consensus.md
  covers: FT-001
  adrs:   ADR-002

DOC-005  Resource Type Reference        reference  docs/reference/resource-types.md
  covers: FT-006, FT-007, FT-008, FT-009
  adrs:   ADR-007, ADR-049, ADR-050
```

---

## Session Tests

```
# Basic validation
ST-200  doc-location-valid-file-parses-correctly
ST-201  doc-e018-location-file-not-found
ST-202  doc-orphaned-emits-w001
ST-203  doc-linked-feature-bidirectional

# Staleness detection
ST-204  w025-fires-when-doc-not-updated-since-completion-tag
ST-205  w025-clear-when-doc-updated-after-completion-tag
ST-206  w025-clear-when-feature-not-yet-verified
ST-207  w025-suppressed-by-status-current-change
ST-208  w027-fires-when-doc-updated-after-tag-no-reverify
ST-209  w027-clear-when-feature-reverified-after-doc-change

# Completeness
ST-210  w026-fires-complete-feature-no-docs
ST-211  w026-advisory-by-default
ST-212  w026-e-class-when-require-on-complete-true

# Commands
ST-213  doc-list-returns-all-docs
ST-214  doc-list-filter-by-type
ST-215  doc-list-stale-matches-w025
ST-216  doc-show-includes-staleness-status
ST-217  doc-bundle-includes-file-content
ST-218  doc-bundle-includes-linked-feature-context
ST-219  doc-bundle-includes-diff-since-last-completion-tag

# Request integration
ST-220  request-create-doc-validates-file-exists
ST-221  request-create-doc-e018-when-file-missing
ST-222  request-change-doc-status-suppresses-w025

# Verify pipeline integration
ST-223  verify-pipeline-stage2-includes-doc-warnings
ST-224  impact-analysis-includes-doc-artifacts
```

---

## Invariants

- E018 fires on `product request validate` — a DOC artifact cannot be created
  pointing to a non-existent file. Create the file first.
- W025 and W027 are mutually exclusive for any given DOC artifact at any
  given time. W025 fires when the feature is ahead of the docs; W027 fires
  when the docs are ahead of the feature.
- `product doc bundle` always includes the full file content. If the file is
  large (> 50KB), Product emits a note that the bundle may exceed typical LLM
  context windows, and offers `--section HEADING` to scope to a section.
- `status: current` set via a change request suppresses W025 until the next
  completion tag is created for any linked feature. It is not a permanent
  suppression.
- Documentation artifacts never block feature completion by default.
  `require-on-complete = true` is an explicit opt-in.
