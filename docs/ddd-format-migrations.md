# DDD Entry Format Migrations

Per PRD §6, every schema change is a format-version bump with a migration
note, and `ddd validate` checks each entry against the version it declares —
existing entries never break silently. This file is the migration record.

## Format 2 (M2)

Format 1 entries remain valid unchanged; nothing forces a migration. Format 2
exists for the two capabilities `ddd report escapes` needs.

### Claims: optional `revalidate_by`

A format-2 claim may declare a revalidation cadence — the date by which its
status must be rechecked:

```yaml
format: 2
id: DDD-cat-01
# ...
changed: 2026-08-02
revalidate_by: 2027-02-01
```

`ddd report escapes` flags any live claim whose `revalidate_by` has passed.
Declaring the field under `format: 1` is a validation violation.

**Migrating a v1 claim:** set `format: 2` and add `revalidate_by` if the claim
should carry a cadence. No other field changes meaning.

### Decisions: pinned `based_on` edges (PRD §6 rule 6)

A format-2 decision pins **every** `based_on` edge with the claim's `status`
and `changed` values as they were at decision time:

```yaml
format: 2
id: dec/cs/async-config
# ...
based_on:
  - claim: DDD-cat-01
    status: reported
    changed: 2026-08-02
```

Basis-loss detection (`ddd report escapes`) compares each pin against the
claim's current `status`/`changed`; a mismatch means the ground the decision
stood on has moved. Format-1 decisions keep plain claim-id strings — valid,
but their basis loss is not detectable, and pinned edges under `format: 1`
are a validation violation. Mixing pinned and plain edges in one format-2
decision is also a violation: pin all edges or stay on format 1.

**Migrating a v1 decision:** set `format: 2` and replace each `- <claim-id>`
with a `claim`/`status`/`changed` map, copying the status and changed values
the claim had **when the decision was made** (from git history if the claim
has since moved; if you re-affirm the decision against the claim's current
state, use today's values — that is a re-decision, note it in `notes`).

### Config: `diff` and `detect` sections

`.ddd/config.yaml` gains two optional sections under `format: 2`:

```yaml
format: 2
intercept: warn
ignore: []

# Per-finding treatment for `ddd diff`: error (default) | warn | off.
diff:
  ungoverned: error
  stale: warn
  uncited_suppression: error

# SARIF files ingested by default; `--sarif` appends.
detect:
  sarif:
    - artifacts/csharp.sarif
    - artifacts/bicep.sarif
```

Using either section under `format: 1` is a validation violation. A v1
config without them stays valid.
