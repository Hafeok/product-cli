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

## Format 4 (M5)

Formats 1–3 remain valid unchanged. Format 4 exists for one capability the
closure-claim seed needs.

### Claims: `version_index`

A closure claim reports what an arrangement closes, and an arrangement is a
versioned thing: `Nullable` closes what it closes *in a given C# version, on
a given SDK*. Before format 4 that anchor could only live in `evidence`
prose, where nothing can read it. A format-4 claim may index it:

```yaml
format: 4
id: DDD-cs-nrt-01
version_index:
  language: "C# 12"
  sdk: "8.0.413"
  target: net8.0
  tool: "Microsoft.CodeAnalysis.NetAnalyzers 9.0.0"
status: reported
```

Every sub-field is optional, because the anchor differs by claim: a language
rule pins `language`, an SDK behaviour pins `sdk`, an analyzer finding pins
`tool`. But an index must anchor **something** — an index with no version in
it is a validation violation, since it would record the appearance of a drift
anchor without the substance. Declaring `version_index` under format 1–3 is
a violation.

The field is what a promoting repo checks before inheriting an entry
(`dec/ddd/seed-lands-in-claims-not-shared`): a closure claim indexed to an
SDK the destination does not run is worse than no entry, because it carries
the catalog's authority without holding there.

## Format 7 (the reopen edge, 2026-08-13)

Formats 1–6 remain valid unchanged. Format 7 exists for one ruling, and
adopts into this store the one shape the ledger format specifies at spec
v1.5 / `format: 4` (`ledger-format-v1.md` §3.7) — the same one-shape rule
the M8 `contract:` scheme followed.

### Decisions: `revisit_if`, the reopen edge

The principal ruled (2026-08-13) that a **watched-not-grounding edge is a
distinct edge type, not a basis**: this claim's death reopens the decision;
it is not the decision's ground.

```yaml
format: 7
based_on:
  - type: mandate
    statement: >-
      The principal's ruling settling PRD §14 question 2 …
revisit_if:
  - claim: DDD-adapter-02
    status: projected
    changed: 2026-08-06
    content: sha256:…
```

Rules that arrive with it:

- **A reopen edge is never a basis.** It lives in its own field, carries its
  own type, and is scanned by its own pass. `based_on`'s basis-loss scan
  cannot see it and `ddd why` renders it under its own heading, below the
  ground and visibly apart. Declaring `revisit_if` under formats 1–6 is a
  violation.
- **Every reopen edge pins**, `content:` included — an unpinned tripwire
  cannot fire, so an edge without one is a violation. The pin is the
  claim's canonical content hash (`ddd.claim-content.v1`), exactly as a
  format-6 claim basis pins.
- **One claim is ground or tripwire, never both.** A claim appearing in
  both lists on one decision is a violation: an edge read both ways is
  precisely the conflation the distinct type exists to end.
- **It reports as `reopen`, never as basis loss.** `ddd report escapes`
  gains a fourth section with its own heading and its own message: a fired
  tripwire means the decision is due a fresh look, its ground untouched.
  A lost basis means the ground moved. Merging the two tells the reader
  neither.
- **It is hashed content.** `revisit_if` joins `decision_content_hash`
  under its own key, so re-typing an edge from ground to tripwire moves the
  decision's content hash — which is what makes the re-decision visible to
  the ledger entry pinning it rather than a silent rewrite.

**Migrating a format-5/6 decision:** set `format: 7`, move the claim edge
out of `based_on` into `revisit_if`, and carry its pin across unchanged
(`status`, `changed`, and the `content` hash at that pinned state — the M8
migration's recovered hashes are the ones already in the ledger's `watched:`
tokens). Do **not** re-pin at the claim's current state: a recorded drift is
the record that the movement was looked at, and re-pinning would erase it.
Then file a new ledger version for the decision, re-taking its
`ddd-content:` pin at the entry's new content and stating the reopen edge —
a re-decision filed for the principal's acceptance, never a silent rewrite.
`ddd content-hash <id>` prints the pin to state.

**Migrated by this amendment:** the three edges the M8 migration carried as
provisional `watched:` markers — `DDD-adapter-02` on
`dec/ddd/internal-not-surface`, `DDD-gates-01` on `dec/rust/no-unwrap`,
`DDD-adapter-01` on `dec/ddd/m6-proceeds-no-flip`. The third is the one
whose pin has drifted; it fires, now as a reopen finding.

## Format 6 (M8, 2026-08)

Formats 1–5 remain valid unchanged. Format 6 exists for the two M8
capabilities: declarations that sign the change they discharge, and
basis loss as an exact content comparison. Both hash forms ride the
ledger's canonical-JSON law (`ledger_core::canon`'s primitives) under
their own domain prefixes — one canonicalisation scheme, per-payload
field sets.

### Seams: signed `bindings` (seam format 2)

A seam declaration's entry family moves to `format: 2` when it carries
`bindings` — the signed transitions the declaration discharges:

```yaml
format: 2
id: seam/rust/added
# ...
bindings:
  - symbol: added
    file: src/lib.rs
    before: sha256:…          # parent-state content hash, or `absent`
    after: sha256:…           # proposed-state content hash
    base_revision: <commit>   # where the parent state is committed
    hash: sha256:…            # ddd.seam-binding.v1 over the five fields
```

A binding's parent state must be committed — a dirty file refuses to
bind (M8 ruling 2) — and matching is by signature only: same-session
matching is retired (M8 ruling 4, closing `DDD-arch-09`). Declaring
`bindings` under seam format 1 is a validation violation; a stored
binding hash that does not recompute is a violation.

### Decisions: content-hash pins

A format-6 decision pins **every** claim edge with `content:` — the
hash of the claim's canonical content at decision time
(`ddd.claim-content.v1`). Basis loss becomes pinned-hash ≠ current-hash,
mechanically exact (spec invariant 2); status and `changed` stay on the
pin as the human-readable record, and since both are part of the hashed
content, hash equality subsumes the old heuristic:

```yaml
format: 6
based_on:
  - type: claim
    claim: DDD-arch-08
    status: reported
    changed: 2026-08-11
    content: sha256:…
```

Declaring `content` under formats 1–5 is a violation; a format-6
decision with a claim pin missing `content` is a violation.

**Migrating a v2–v5 decision:** set `format: 6` and add `content:` to
every claim pin. The pinned hash is the claim's content *at decision
time*: when the claim's current status+changed still equal the pin,
hash the current content; when the pin records drift that was
deliberately kept (a re-affirmed edge), recover the claim's content at
the pinned state from git history and hash that — the recorded drift
then keeps reporting as basis loss, exactly as before, now exactly.

## Format 5 (PRD review, 2026-08)

Formats 1–4 remain valid unchanged. Format 5 exists for one ruling: a
decision's basis becomes **typed** — `claim | constraint | mandate |
preference | experiment | risk-acceptance`. Forcing claim-basis where none
exists manufactures weak claims to pass validation, the exact corruption
the graph exists to prevent; the non-claim types are honest first-class
bases instead. The `basedOn → claim` edge remains the load-bearing form
where a claim *is* the basis.

### Decisions: typed `based_on` entries

A format-5 decision types **every** basis. A claim basis is the format-2
pinned shape plus `type: claim` — the pin discipline (rule 6) carries
forward unchanged. A non-claim basis states what it rests on; a
`risk-acceptance` basis refs the record it rests on (which must exist in
`decisions/` with `kind: risk-acceptance` — checked by the ontology rules):

```yaml
format: 5
id: dec/x/y
# ...
based_on:
  - type: claim
    claim: DDD-cat-01
    status: reported
    changed: 2026-08-02
  - type: preference
    statement: Four documents beat one implementable-contract PRD
  - type: risk-acceptance
    ref: risk/rule/ca2007
```

Ontology rule 3 is restated accordingly: "every decision has ≥1 `basedOn`
claim" becomes **"every decision has ≥1 typed basis"** — satisfied by a
claim edge or by a format-5 non-claim basis. `ddd why` renders the basis
type; basis-loss detection (`ddd report escapes`) still checks claim pins
and counts non-claim bases separately (they carry no status that could
move). Declaring a type under formats 1–4, leaving a basis untyped under
format 5, a `type: claim` basis without its pin, and a non-claim basis
with no `statement:` (or, for `risk-acceptance`, no `ref:`) are each
validation violations.

**Migrating a v1–v2 decision:** nothing forces a migration — existing
entries stay valid under their declared format, and their untyped edges
*read as* `type: claim` implicitly (that is what they always were; the
plan-gate call here is no mechanical rewrite, so decision-time records
stay byte-identical). Move to `format: 5` only when filing or re-filing a
decision whose basis is genuinely not a claim — that is a re-decision of
its basis, note it in `notes`.

**Migrating a v1–v3 claim:** set `format: 4` and add `version_index` if the
claim is a closure finding whose truth depends on a toolchain version. Claims
about method, architecture or scope need no index and should stay at their
current format. No other field changes meaning.

### Config: the pair map plus the web detection sources (M7)

`.ddd/config.yaml` gains, at `format: 4`:

- **`pair`** — the governed HTML+CSS pair map (`units`, each an
  `html`/`css` glob pair) plus the class-contract thresholds
  (`ignore_classes`, decorative one-off class globs). Consumed by the
  `htmlcss` adapter's enrichment and by the pair contract check in
  `ddd report escapes`. Thresholds are data; what a pair *means* stays in
  the adapter.
- **`detect.stylelint` / `detect.htmlvalidate`** — the tools' own JSON
  output files as emitted sources for the `stylelint` / `htmlvalidate`
  namespaces (stylelint ≥16 writes its report to stderr:
  `stylelint <files> -f json 2> out.json`; html-validate:
  `html-validate -f json <files> > out.json`). Configured sources need no
  registration — `.stylelintrc.json` and `.htmlvalidate.json` are found by
  the detection walk like `bicepconfig.json` is.
- **`detect.tokens`** — design-token stylesheets whose custom properties
  become the configured source for the `tokens` namespace, one rule per
  token, so a token resolves through `why` to its decision and `diff`
  flags UNGOVERNED or STALE tokens.

Declaring any of these under `format: 1`–`3` is a validation violation.
Existing configs remain valid unchanged.
