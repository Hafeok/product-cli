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

## Format 3 (M3/M4)

Formats 1 and 2 remain valid unchanged. Format 3 exists for the two
capabilities the MCP surface (M3) and the interceptor (M4) need.

### Predicates: `obligations` (pattern predicates)

A format-3 predicate may declare the obligation list a pattern instance
must answer at declaration time (`ddd_declare_pattern` rejects unanswered
obligations):

```yaml
predicate:
  id: pred/composition/decorator
  format: 3
  # ...
  obligations:
    - ordering
    - identity-preservation
    - forwarding-completeness
```

Declaring `obligations` under format 1 or 2 is a validation violation.

### Config: `intercept_by_class` and `adapter` sections

`.ddd/config.yaml` gains two optional sections under `format: 3`:

```yaml
format: 3
intercept: warn            # the global default, as before

# Per-artifact-class overrides (PRD §8): enforce | warn | off.
intercept_by_class:
  code: enforce            # C# edits
  configuration: warn      # Bicep edits

# Per-language adapter switches, keyed by adapter language name.
adapter:
  csharp:
    internal_is_surface: false        # dec/ddd/internal-not-surface; flip in library repos
    exported_attributes: [McpServerTool]
    command: ["roslyn-language-server", "--stdio", "--autoLoadProjects"]  # host override
  bicep:
    command: ["bicep-ls"]
```

Both intercept vocabularies are validated (`enforce | warn | off`);
unknown modes are violations. Declaring either section under a lower
format is a violation.

### Seam events (new entry family, format 1)

The interceptor logs every classified surface outcome as one row under
`.ddd/seams/events/` — separate from the seam *declarations* directly in
`.ddd/seams/`. Rows are format-versioned like every other entry and load
into `validate`. This is the correspondence dataset (PRD §8); its field
set is the schema M5 files claims against.
