# Product Request Builder Specification

> Standalone reference for the interactive request builder — the human-facing
> incremental interface to the request model.
>
> The builder is a convenience layer over the request YAML format.
> The YAML is always the source of truth. Agents skip the builder entirely.
>
> Amendment to ADR-032 in `product-adrs.md`.

---

## Overview

The request model (ADR-032) has two interfaces:

**Direct YAML** — produce the full request YAML in one shot, validate, apply.
This is the agent path. An LLM reasons about the full intent and emits complete YAML.

**Builder** — start a draft, add artifacts or mutations one at a time, get
immediate feedback after each addition, validate progressively, submit when done.
This is the human-at-terminal path.

Both paths produce identical YAML and use the same `product request apply` under
the hood. The builder is not a separate system — it is an incremental editor for
the request format with a feedback loop after each step.

---

## Draft Lifecycle

A builder session is a draft stored at `.product/requests/draft.yaml` (gitignored).
One active draft per working directory at a time.

```
product request new create    →  creates draft.yaml, type: create
     ↓
product request add [...]     →  appends to draft.yaml, validates incrementally
     ↓  (repeat)
product request status        →  human-readable summary of current draft
product request validate      →  full cross-artifact validation
     ↓
product request submit        →  apply draft.yaml atomically, archive it
     or
product request discard       →  delete draft.yaml, abandon session
```

---

## Starting a Session

### `product request new create`

```
product request new create

  Draft started: .product/requests/draft.yaml
  Type: create

  Add artifacts with:
    product request add feature
    product request add adr
    product request add tc
    product request add dep
    product request add doc

  See current state:  product request status
  Full validation:    product request validate
  Apply when ready:   product request submit
  Abandon:            product request discard
```

### `product request new change`

```
product request new change

  Draft started: .product/requests/draft.yaml
  Type: change

  Target existing artifacts with:
    product request add target FT-001
    product request add target ADR-002

  See current state:  product request status
  Full validation:    product request validate
  Apply when ready:   product request submit
  Abandon:            product request discard
```

### Existing draft

If a draft already exists when `product request new` is called:

```
product request new create

  ⚠ An active draft already exists: .product/requests/draft.yaml

  Options:
    product request status    — see current draft
    product request submit    — apply and start fresh
    product request discard   — abandon and start fresh
    product request continue  — resume the existing draft (default)
```

---

## `product request add` — Create Mode

Each `add` command prompts for required fields, runs immediate structural
validation on the new artifact in context of the current draft and graph,
and appends to `draft.yaml`.

All fields can be provided as flags to skip prompts — useful for scripting or
when the user knows exactly what they want:

```bash
product request add feature --title "Rate Limiting" --phase 2 --domains "api,security"
```

### `product request add feature`

```
product request add feature

  Title: Rate Limiting
  Phase [1]: 2
  Domains (vocabulary: api security networking iam consensus storage ...): api security

  ✓ Feature added: ref:ft-rate-limiting
  ⚠ W002: No ADRs linked yet.
    Add with: product request add adr
```

Flags: `--title`, `--phase`, `--domains`, `--depends-on`

After adding:
- Checks domain vocabulary (E012 if unknown)
- Warns if no ADRs in draft yet (W002)
- Shows the assigned `ref:` name for use in subsequent steps

### `product request add adr`

```
product request add adr

  Title: Token bucket algorithm for rate limiting
  Domains: api
  Scope [domain]: domain
  Link to feature? [ft-rate-limiting]: y

  ✓ ADR added: ref:adr-token-bucket
  ✓ Cross-linked to ft-rate-limiting
```

Flags: `--title`, `--domains`, `--scope`, `--features`, `--governs`, `--supersedes`

After adding:
- Validates domain vocabulary
- Validates scope value
- Checks if an existing accepted ADR conflicts (advisory — reports G005 candidates)
- Reports cross-links added to linked features

### `product request add tc`

```
product request add tc

  Title: Rate limit enforced at 100 req/s
  Type [scenario]: scenario
  Level [component]: integration
  Link to feature? [ft-rate-limiting]: y
  Link to ADR? [adr-token-bucket]: y
  Runner [bash]: bash
  Runner args: scripts/test-harness/rate-limit.sh

  ✓ TC added: ref:tc-rate-limit
  ✓ Linked to ft-rate-limiting, adr-token-bucket
```

Flags: `--title`, `--tc-type`, `--level`, `--features`, `--adrs`, `--runner`,
       `--runner-args`, `--runner-timeout`, `--requires`

After adding:
- Validates type (E006 if unknown)
- Validates level (E006 if unknown)
- If `type: invariant` or `type: chaos` — reminds to add formal block in body
- If `type: exit-criteria` — notes this affects the phase gate

### `product request add dep`

```
product request add dep

  Title: Redis
  Type [service]: service
  Version: >=7
  Governing ADR — create new or link existing? [new]: new
  ADR title: Redis for rate limit state

  ✓ DEP added: ref:dep-redis
  ✓ Governing ADR added: ref:adr-redis-choice
  ✓ E013 satisfied — dep has governing ADR in draft
```

Flags: `--title`, `--dep-type`, `--version`, `--adr`, `--availability-check`,
       `--breaking-change-risk`

After adding:
- Immediately checks E013 (dep without governing ADR) — the `new` option creates
  a stub ADR in the same step, satisfying the constraint before it can fire
- If linking to an existing ADR: validates the ADR exists and governs nothing else

### `product request add doc`

```
product request add doc

  Title: Rate Limiting API Reference
  Type [reference]: reference
  Format [markdown]: markdown
  Location: docs/reference/rate-limiting.md

  Checking file exists... ✓  docs/reference/rate-limiting.md found
  Link to feature? [ft-rate-limiting]: y

  ✓ DOC added: ref:doc-rate-limit-ref
  ✓ Linked to ft-rate-limiting
```

Flags: `--title`, `--doc-type`, `--format`, `--location`, `--features`, `--adrs`

After adding:
- Validates `location` file exists (E018 if not — prompted to create it first)

---

## `product request add` — Change Mode

### `product request add target`

Adds a target artifact and opens an interactive mutation builder for it:

```
product request add target FT-001

  Target: FT-001 — Cluster Foundation [complete, phase 1]

  Current values:
    domains: [consensus, networking]
    adrs: [ADR-001, ADR-002, ADR-006]
    status: complete

  Add mutation:
    1  append to array field
    2  set scalar field
    3  remove from array field
    4  delete optional field
    5  done (no more mutations for this target)

  Choice [1]: 1
  Field: domains
  Value: security

  ✓ Mutation added: FT-001 / append / domains / security
  ⚠ W010: FT-001 has no acknowledgement for security domain.
    Add acknowledgement? [y]: y
  Reason: Rate limiting is not an auth boundary — security handled by ADR-015

  ✓ Acknowledgement mutation added: FT-001 / set / domains-acknowledged.security

  Add another mutation for FT-001? [n]: n
  Add another target? [y]: n
```

Flags: `--field`, `--op`, `--value`

After each mutation:
- Validates the field name exists in the artifact schema
- Validates the value type and vocabulary
- Runs relevant W-class checks (W010 for domain without acknowledgement, etc.)
- Suggests follow-up mutations when needed (acknowledgement, reverse links)

### `product request add acknowledgement`

Shortcut for the common case of acknowledging a domain gap:

```
product request add acknowledgement FT-001 security \
  "Rate limiting is not an auth boundary — handled by ADR-015"

  ✓ Mutation added: FT-001 / set / domains-acknowledged.security
  ✓ W010 resolved for FT-001 security
```

---

## `product request status`

Human-readable summary of the current draft. Never raw YAML — that's `product request show`.

```
product request status

  Draft: create  (5 artifacts, 1 warning)
  ──────────────────────────────────────────────────────────────────
  ✓  Feature    ref:ft-rate-limiting      Rate Limiting
                  phase 2  ·  domains: api security
                  adrs → adr-token-bucket, adr-redis-choice
                  tests → tc-rate-limit
                  deps → dep-redis

  ✓  ADR        ref:adr-token-bucket      Token bucket algorithm
                  domain  ·  api
                  ← ft-rate-limiting

  ⚠  ADR        ref:adr-redis-choice      Redis for rate limit state
                  domain  ·  api
                  governs dep-redis
                  ← ft-rate-limiting
                  No TC linked — add with: product request add tc

  ✓  TC         ref:tc-rate-limit         Rate limit at 100 req/s
                  scenario  ·  integration  ·  bash runner
                  ← ft-rate-limiting  ← adr-token-bucket

  ✓  DEP        ref:dep-redis             Redis
                  service  ·  >=7
                  governed by adr-redis-choice
                  ← ft-rate-limiting

  ──────────────────────────────────────────────────────────────────
  Warnings: 1  (W002: adr-redis-choice has no linked TC)
  Run: product request validate   — full cross-artifact check
  Run: product request submit     — validate and apply
```

Status indicators:
- `✓` — artifact is structurally valid so far
- `⚠` — artifact has warnings (W-class)
- `✗` — artifact has errors (E-class) — submit will be blocked

---

## `product request validate`

Runs the full cross-artifact validation — same as `product request validate draft.yaml`.
Shows all findings across the complete draft at once.

```
product request validate

  Validating draft (5 artifacts)...

  ⚠ W002  adr-redis-choice has no linked TC
    Suggestion: product request add tc --adrs adr-redis-choice

  ✓ No E-class findings.

  Ready to submit with warnings.
  Run: product request submit
```

---

## `product request submit`

Validates the draft completely, then applies it. Identical to
`product request apply .product/requests/draft.yaml` except that on success it
archives the draft rather than leaving it.

```
product request submit

  Validating...  ⚠  1 warning (W002: adr-redis-choice has no linked TC)

  Submit anyway? [y]: y

  Applying:
    FT-009  Rate Limiting                     [new feature]
    ADR-031 Token bucket algorithm            [new ADR]
    ADR-032 Redis for rate limit state        [new ADR]
    TC-050  Rate limit at 100 req/s           [new TC]
    DEP-007 Redis                             [new dep]

  Graph check...  ✓  clean

  Done. 5 artifacts created.
  Draft archived: .product/requests/archive/2026-04-14T09-14-22-draft.yaml
  Run `git push --tags` after product verify FT-009.
```

On E-class errors, submit refuses:

```
product request submit

  Validating...  ✗  1 error

  error[E013]: dep-redis has no governing ADR in draft or existing graph
    location: artifacts[3] (dep-redis)

  Fix the error before submitting.
  Run: product request add adr --governs dep-redis
```

---

## `product request edit`

Opens `draft.yaml` in `$EDITOR` directly. For users who prefer to write YAML
and want the builder's lifecycle management (draft tracking, validation on
status, archive on submit) without the interactive prompts.

```
product request edit
# → opens .product/requests/draft.yaml in $EDITOR
```

After editing, `product request status` shows the updated state with full
incremental validation applied to the new content.

---

## `product request show`

Prints the raw draft YAML to stdout. For piping or inspection.

```
product request show
# → prints .product/requests/draft.yaml to stdout

product request show | product request validate --stdin
# → validate from stdin (same as validate FILE)
```

---

## `product request continue`

Resumes an existing draft session, showing its current status.

```
product request continue

  Resuming draft: .product/requests/draft.yaml (started 2026-04-14T09:00:00)
```

Equivalent to running `product request status` — the command exists as a
clear entry point for returning to an interrupted session.

---

## `product request discard`

Abandons the current draft. Asks for confirmation.

```
product request discard

  Discard current draft? This cannot be undone.
  Draft has 3 artifacts. [y/N]: y

  Draft discarded.
```

With `--force`:

```
product request discard --force
# → no confirmation prompt
```

---

## Command Summary

```bash
# Session management
product request new create              # start a create session
product request new change              # start a change session
product request continue                # resume existing draft
product request discard [--force]       # abandon draft

# Building
product request add feature [FLAGS]     # add a feature artifact
product request add adr [FLAGS]         # add an ADR artifact
product request add tc [FLAGS]          # add a TC artifact
product request add dep [FLAGS]         # add a dependency artifact
product request add doc [FLAGS]         # add a documentation artifact
product request add target ID [FLAGS]   # add a change target (change mode)
product request add acknowledgement ID DOMAIN REASON  # shortcut

# Inspection
product request status                  # human-readable draft summary
product request show                    # raw YAML
product request validate                # full cross-artifact validation
product request diff                    # show what would change on submit

# Submission
product request submit [--force]        # validate and apply
product request edit                    # open $EDITOR on draft.yaml
```

---

## Draft File Format

The draft is standard request YAML — no special builder metadata. Opening it
in an editor or passing it directly to `product request apply` works identically
to what the builder produces through prompts.

```yaml
# .product/requests/draft.yaml
# Created: 2026-04-14T09:00:00Z
# Builder session — submit with: product request submit

type: create
reason: "Add rate limiting to the resource API"
artifacts:
  - type: feature
    ref: ft-rate-limiting
    title: Rate Limiting
    phase: 2
    domains: [api, security]
    adrs: [ref:adr-token-bucket, ref:adr-redis-choice]
    tests: [ref:tc-rate-limit]
    uses: [ref:dep-redis]

  - type: adr
    ref: adr-token-bucket
    title: Token bucket algorithm for rate limiting
    domains: [api]
    scope: domain
    features: [ref:ft-rate-limiting]

  - type: tc
    ref: tc-rate-limit
    title: Rate limit enforced at 100 req/s
    tc-type: scenario
    level: integration
    runner: bash
    runner-args: ["scripts/test-harness/rate-limit.sh"]
    validates:
      features: [ref:ft-rate-limiting]
      adrs: [ref:adr-token-bucket]

  - type: dep
    ref: dep-redis
    title: Redis
    dep-type: service
    version: ">=7"
    adrs: [ref:adr-redis-choice]
    availability-check: "redis-cli ping"

  - type: adr
    ref: adr-redis-choice
    title: Redis for rate limit state
    domains: [api]
    scope: domain
    governs: [ref:dep-redis]
    features: [ref:ft-rate-limiting]
```

---

## Archive

On successful submit, the draft is moved to:

```
.product/requests/archive/2026-04-14T09-14-22-draft.yaml
```

The archive directory is gitignored. Archives are local history — not committed,
not shared. Useful for reviewing what you submitted in a session.

```bash
ls .product/requests/archive/   # list past submitted drafts
```

---

## `product.toml` Configuration

```toml
[request-builder]
# Prompt for confirmations by default. Set false for non-interactive use.
interactive = true

# Open $EDITOR on 'product request edit'. Defaults to $EDITOR env var.
# editor = "vim"

# Warn when submitting with W-class findings (default: warn and prompt)
# "always" = always submit without prompting
# "warn"   = prompt when warnings exist (default)
# "block"  = treat W-class as E-class, refuse to submit
warn-on-warnings = "warn"
```

---

## Session Tests

```
# Session lifecycle
ST-250  new-create-starts-draft
ST-251  new-change-starts-draft
ST-252  new-with-existing-draft-prompts-options
ST-253  continue-resumes-existing-draft
ST-254  discard-removes-draft
ST-255  discard-force-no-confirmation
ST-256  submit-archives-draft-on-success

# Add commands — create mode
ST-257  add-feature-prompts-required-fields
ST-258  add-feature-flags-skip-prompts
ST-259  add-feature-invalid-domain-emits-e012
ST-260  add-feature-w002-no-adrs-yet
ST-261  add-adr-cross-links-to-feature
ST-262  add-tc-validates-type-and-level
ST-263  add-dep-creates-governing-adr-in-same-step
ST-264  add-dep-e013-satisfied-within-draft
ST-265  add-doc-validates-location-exists
ST-266  add-doc-e018-when-file-missing

# Add commands — change mode
ST-267  add-target-shows-current-values
ST-268  add-target-mutation-validates-field
ST-269  add-target-suggests-acknowledgement-on-w010
ST-270  add-acknowledgement-shortcut
ST-271  add-target-unknown-field-emits-e006

# Status and validation
ST-272  status-shows-all-artifacts-with-indicators
ST-273  status-shows-warning-count
ST-274  status-shows-error-count
ST-275  validate-runs-full-cross-artifact-check
ST-276  validate-reports-all-findings-not-just-first

# Submit
ST-277  submit-validates-before-applying
ST-278  submit-blocked-on-e-class-error
ST-279  submit-prompts-on-warnings
ST-280  submit-force-skips-warning-prompt
ST-281  submit-applies-identically-to-request-apply

# Edit and show
ST-282  edit-opens-draft-in-editor
ST-283  show-prints-raw-yaml
ST-284  show-output-valid-for-request-validate

# YAML equivalence
ST-285  builder-output-identical-to-hand-written-yaml
ST-286  hand-written-yaml-applyable-without-builder
```

---

## Invariants

- The builder never applies a partial draft. `product request submit` runs full
  validation before any file is written. If validation fails, the draft is
  unchanged.
- The draft file is always valid request YAML. At every point in the session,
  `product request validate draft.yaml` produces consistent results with
  `product request validate` (the builder command).
- The builder produces no output that the direct YAML path cannot also produce.
  There is no builder-only feature. Every operation maps to a YAML construct.
- Incremental validation after each `add` command is structural only (no LLM,
  no git operations). It is always faster than 100ms.
- `product request submit` is exactly `product request apply .product/requests/draft.yaml`
  plus archiving. The application semantics are identical.
- One active draft per working directory. Starting a new session when one exists
  always surfaces the existing draft rather than silently overwriting it.
