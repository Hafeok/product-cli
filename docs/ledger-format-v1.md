# Decision Ledger — Entry Format v1

**Status:** normative for `format: 1`.
**Scope:** L0 of `decision-ledger-prd.md` — the file format, the canonical
form, the version hash, and the `verify` gate. No graph, no index, no merge,
no coverage query, no federation.
**Audience:** an implementer building a second implementation of this format,
working from this document alone. Where this document and the reference
implementation (`ledger-core`) disagree, this document wins.

A companion, `ledger-format-migrations.md`, is the migration record: every
schema change is a `format` bump with a note there, and validation is always
against the version an entry declares.

---

## 1. What the format is for

A decision has identity independent of any repository, content-addressed
versions, and an acceptance that signs a **hash**, never an id. That is the
property that makes acceptance mean something: it names an exact state, not
"whatever this decision currently says". Every rule below exists to keep that
property true across machines, platforms, and independent implementations.

Two versions carry the load:

| Version | Governs | Bump when |
|---|---|---|
| `format` | how a *file* is read | a field is added, removed, or re-shaped |
| `CANONICAL_FORM` (§4) | how a *hash* is computed | a hashed field changes meaning |

They are independent. A `format` bump that leaves hashed semantics alone
keeps every existing acceptance valid. A `CANONICAL_FORM` bump does not, and
is therefore a governed act, not a fix.

---

## 2. Storage layout

```
.decisions/
  sets/<set-id>.yml           declared scope: floor, ground, owner
  log/<changeset-ulid>.yml    append-only; the source of truth
  index/                      gitignored; rebuildable cache (not written at L0)
```

- Files are read with either `.yml` or `.yaml`; writers emit `.yml`.
- A log file is written once and **never edited**. A correction is a new
  version; a reversal is a revocation.
- The file stem must equal the id it declares — `<ulid>.yml` for a change-set
  whose `id` is `cs:<ulid>`, `<set-id>.yml` for a set. A disagreement is a
  schema fault.
- `index/` is not created at L0. The ignore line exists so an L2 rebuild
  cache can never be committed by accident.

---

## 3. Schemas

Every file declares `format: 1`.

### 3.1 Identifiers

```
dec:<namespace>/<ulid>     a decision, stable forever
cs:<ulid>                  a change-set
acc:<ulid>                 an acceptance
sha256:<64 lowercase hex>  a version hash
```

- **ULID**: 26 characters of Crockford base32 (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`
  — `I`, `L`, `O`, `U` excluded). The first character must not exceed `7`;
  above that overflows the 48-bit millisecond timestamp field. Offline
  generatable, lexicographically sortable by creation time.
- **Namespace**: an *owning scope*, not a repo path. Dot-separated segments of
  lowercase alphanumerics and dashes. Repos may reference decisions in
  namespaces they do not own.
- A decision id is permanent. Supersession mints a new id carrying a
  `supersedes` edge; it never mutates or reuses one.
- A decision's namespace is not restated as a separate field: it is inside
  the id, and a second spelling of one fact is a second thing that can
  disagree.

### 3.2 Identity — the actor an acceptance names

An identity is an **email address**, normalised to lowercase at parse.
`§4.4` of the PRD requires `accepted-by` to *resolve* to a human identity; an
address resolves, a display name decorates. Comparing addresses also makes
the blame check (§5, `L009`) robust against the punctuation and whitespace
drift real `user.name` values carry.

Requirements: exactly one `@`, non-empty local part and domain, no
whitespace. A dotless domain is legal.

**Model and CI identities are refused as acceptors** (`L006`). An identity is
refused when any of the following holds:

1. it is a listed vendor no-reply address (`noreply@anthropic.com`,
   `noreply@openai.com`, `noreply@github.com`, `noreply@google.com`);
2. it contains the literal `[bot]`;
3. its domain is `github-actions.*`;
4. its local part is one of `actions, automation, bot, build, cd, ci,
   dependabot, do-not-reply, github-actions, gitlab-ci, jenkins, no-reply,
   noreply, renovate, robot`;
5. its local part contains, as a **whole token** (split on non-alphanumerics,
   trailing digits stripped), one of `agent, ai, aider, chatgpt, claude,
   codex, copilot, cursor, devin, gemini, gpt, llama, llm, mistral, model`.

Whole-token matching is deliberate: `claudia@` and `alain@` are people.
`ai@` is refused and the false positive is accepted knowingly — the person
uses a fuller address; accepting a model is the worse error by a wide margin.

**This list is a floor, not a proof.** It catches the identities a CI system
or an agent harness produces *by default*, which is where the failure
actually occurs. It cannot catch a model configured with a human-looking
address. That gap is closed by review, and by `L009`.

### 3.3 Tolerance

Tiers are ordered `T0 < T1 < T2` (`way-of-working-decision-allocated-delivery.md`
§2.2). A set declares a **floor**. A version pins the floor it was created
under and may carry an up-only `tolerance_override`:

```
effective_tier = tolerance_override, else tolerance_floor_at_creation
```

An override **at or below** the pinned floor is invalid and is rejected at
write, not flagged at review (`L004`). Equality is rejected too: it is not
"above", and it is a no-op that would only add noise to the hash.

The floor is pinned on the version, not read live from the set. Without the
pin the effective tier is not recomputable after a floor raise, so the hash
could not be stable and acceptance-binds-the-tier would be unimplementable.
The consequence is intended: raising a set's floor does not move any existing
hash, so acceptances survive — but every member whose effective tier now
falls below the new floor is **stranded** (`L005`) until a new version pins
the new floor. Entries are never grandfathered.

### 3.4 Discharge pointers

A typed pointer, wire form `scheme:payload`:

| Scheme | Payload | Example |
|---|---|---|
| `analyzer` | rule id | `analyzer:DEC001-no-float-money` |
| `test` | fully-qualified name | `test:ExportIdempotencyTests` |
| `policy` | platform policy id | `policy:deny-public-blob` |
| `whatif` | pre-deployment assertion | `whatif:no-public-network` |
| `otel` | metric name | `otel:dec.004.deadletter` |
| `actor` | an identity (§3.2) | `actor:emk@delegate.dk` |

An unknown scheme is a schema fault: a pointer nothing can ever resolve is
prose, and prose is what this format exists to replace. Nothing *resolves*
these at L0.

`discharge_stage` is one of `pr | dev | staging | prod`, the ground table's
stages. Discharging later than the ground allowed is waste; earlier is
fiction.

### 3.5 Set file — `.decisions/sets/<set-id>.yml`

```yaml
format: 1
id: ledger-design                       # lowercase alphanumerics, dashes, dots
title: Decision Ledger — the L0 settled design
tolerance_floor: T1                     # T0 | T1 | T2
ground: characterised                   # characterised | uncharacterised
owner: emk@delegate.dk
created_at: 2026-08-10
notes: |                                # optional
  free text
```

**A set does not list its members.** A decision-version names its set;
membership is derived by query. Restating membership here would make every
addition a rewrite of a shared file — fighting append-only and conflicting on
every branch — for a denominator that comes out the same either way.

The honest limit, which every coverage report must state: coverage is
measured against the *enumerated* set, and nothing verifies the set itself.

### 3.6 Change-set file — `.decisions/log/<ulid>.yml`

```yaml
format: 1
id: cs:01K2C4YQJ3F8M0PT5W7NZ9RDXV
created_at: 2026-08-10T09:14:22Z        # RFC 3339
created_by: emk@delegate.dk             # who performed the act, not who signs
parents: [cs:01K2C4M...]                # optional
note: What this act was.                # optional

decisions:                              # identity objects, first appearance only
  - id: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    created_at: 2026-08-10T09:14:22Z
    created_by: emk@delegate.dk

versions:
  - decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    parent: sha256:...                  # optional; absent on the first version
    hash: sha256:...                    # the stored digest (§4)
    set: ledger-design
    statement: Monetary amounts use decimal, never double.
    allocation: constraint              # constraint|criterion|judgment|escaped
    discharge: [analyzer:DEC001-no-float-money]
    discharge_stage: pr                 # criterion only
    expectation: "…"                    # required when a discharge is otel:
    actor: emk@delegate.dk              # judgment only
    exposure: "…"                       # escaped only
    accepted_by: emk@delegate.dk        # escaped only
    review_by: 2026-09-01               # escaped only
    tolerance_floor_at_creation: T1
    tolerance_override: T2              # optional, strictly above the pin
    based_on: [prd:decision-ledger-prd#4.2.1]
    supersedes: dec:…                   # optional; no command at L0

acceptances:
  - id: acc:01K2C5…
    decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
    version: sha256:…                   # the hash signed, never the id
    actor: emk@delegate.dk
    at: 2026-08-10T09:20:00Z
    scope: version                      # or class:<discharge-ref>
    expires_at: 2027-08-10              # optional (OD-6 open)
    signature: ""                       # reserved; empty in format 1

revocations:
  - acceptance: acc:01K2C5…
    at: 2026-08-11T09:00:00Z
    by: emk@delegate.dk
    reason: filed against the wrong version
```

**Unknown keys are rejected** in every schema. A field nobody reads reads as
governance that is not there.

Per-allocation obligations, enforced at parse:

| Allocation | Requires | May not carry |
|---|---|---|
| `constraint` | — | anything but `discharge` |
| `criterion` | `discharge` (≥1), `discharge_stage`; `expectation` when any pointer is `otel:` | `actor`, `exposure`, `accepted_by`, `review_by` |
| `judgment` | `actor` | everything else |
| `escaped` | `exposure`, `accepted_by`, `review_by` | everything else |

`allocation` may be **absent** — enumerated-but-unallocated is a real
intermediate state, and a file describing it is well-formed. It is simply not
a shippable state, which is what `L001` says.

`signature` is reserved and must be empty under `format: 1`. It exists so a
cryptographic upgrade (OD-3) is additive rather than a migration; its format
is deliberately unspecified.

`based_on` is a list of single-token basis pointers. Its vocabulary is
**open** at L0 — nothing dereferences a basis pointer yet, and closing the
vocabulary now would reject adopters' existing reference schemes for no gain.
§9.4 of the PRD closes it at L4.

---

## 4. Canonicalisation and hashing

**YAML is the file format; canonical JSON is the hash form.** Routing through
a second, restricted serialisation is what makes "a formatting-only edit
leaves the hash unchanged" structural rather than a rule someone must
remember: quoting style, key order, indentation, comments, line endings and
`null`-versus-absent all disappear in the parse, before anything is hashed.
It also removes YAML's ambiguity — anchors, tags, four multi-line scalar
styles — from the surface a second implementation has to reproduce.

### 4.1 The hashed field set

Exactly these keys, and no others:

```
decision · parent · set · statement · allocation · discharge ·
discharge_stage · actor · expectation · exposure · accepted_by ·
review_by · tolerance_floor_at_creation · tolerance_override ·
based_on · supersedes
```

Outside the hash: the `hash` field itself (including it would be circular),
everything at change-set level (`format`, `id`, `created_at`, `created_by`,
`parents`, `note`), and all acceptances and revocations.

Note that **both tolerance inputs are hashed, not the resolved tier**. `T0`
floor with a `T2` override and a native `T2` floor both resolve to an
effective `T2`, but they are different provenance and must not collide —
which is what keeps override-rate-per-set (PRD §10) computable from hashed
content.

### 4.2 The algorithm

1. Parse the file. Take only the hashed field set.
2. **Normalise every string**, in this order:
   a. replace `\r\n` and lone `\r` with `\n`;
   b. normalise to Unicode NFC;
   c. strip leading and trailing ASCII whitespace
      (`\t \n \v \f \r` and space).
3. **Treat as absent**: a missing key, an explicit `null`, an empty
   collection, and any string that step 2 reduces to the empty string. Absent
   keys are omitted from the object; there is no `null` in the canonical form.
4. **List fields are sets.** `discharge` and `based_on` are rendered as their
   members' canonical string forms, deduplicated, then sorted ascending by
   Unicode code point. Reordering a list in a file is formatting.
5. Emit a JSON object with keys sorted ascending by Unicode code point, with
   no insignificant whitespace and no trailing newline. Keys in this format
   are ASCII, where code-point order coincides with RFC 8785's UTF-16
   code-unit order.
6. Strings are escaped per RFC 8785: the two-character forms `\" \\ \b \f \n
   \r \t` where they exist, `\u00XX` for other control characters, and every
   other character emitted literally as UTF-8.
7. **No floating-point value may appear in hashed content.** A float is a
   schema fault. This removes RFC 8785's entire number-serialisation problem;
   the only numeric field in the format (`format`) is not hashed.
8. Dates are `YYYY-MM-DD`. No timestamp is hashed, so time-zone
   normalisation never arises.

### 4.3 The digest

```
version_hash = "sha256:" + lowercase_hex(
    SHA-256( "ledger.decision-version.v1" ‖ 0x0A ‖ canonical_json_utf8 )
)
```

The prefix is domain separation **and** a version pin. A future entity type
gets its own prefix and can never collide. A `format` bump that changes what
a hashed field *means* must bump the prefix to `.v2`, with a migration note —
otherwise acceptances signed under the old reading would silently re-point.

A short form (first 12 hex characters) exists for display only and is never
compared.

### 4.4 Conformance vector

A second implementation must reproduce both of these exactly.

Input (a version with `parent`, `discharge_stage`, `actor`, `expectation`,
`exposure`, `accepted_by`, `review_by`, `tolerance_override` and `supersedes`
all absent):

```yaml
decision: dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV
set: ledger-design
statement: Monetary amounts use decimal, never double.
allocation: constraint
discharge: [analyzer:DEC001-no-float-money]
tolerance_floor_at_creation: T1
based_on: [prd:decision-ledger-prd#4.2.1]
```

Canonical JSON (one line, shown wrapped):

```
{"allocation":"constraint","based_on":["prd:decision-ledger-prd#4.2.1"],
"decision":"dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV",
"discharge":["analyzer:DEC001-no-float-money"],"set":"ledger-design",
"statement":"Monetary amounts use decimal, never double.",
"tolerance_floor_at_creation":"T1"}
```

Digest:

```
sha256:ac2a68023018391550b542b1f093104f1f32115d3603e2fda9c37805875437cc
```

The eight fixture stores under `ledger-cli/tests/fixtures/` are further
vectors: their stored hashes are real, and a canonicalisation change makes
every one of them fail.

---

## 5. The gate

`ledger verify` fails for a **schema fault** or one of **nine classes**, and
for nothing else. A tenth reason is a change to this document.

### 5.1 The parse gate

`SCHEMA` covers: a file that does not parse against the format it declares;
an unknown `format`; an unknown key; an unknown discharge scheme; a file stem
disagreeing with its declared id; a duplicate id; a per-allocation obligation
from §3.6 that is not met (except the escape's, which is `L002`); a
non-empty `signature`; a version naming an undeclared set; a revocation
naming an acceptance nobody filed.

### 5.2 The nine

| Code | Fails when |
|---|---|
| `L001` | a decision's latest version carries no `allocation` |
| `L002` | an `escaped` version is missing `exposure`, `accepted_by`, or `review_by` |
| `L003` | a live acceptance of a decision's current version has `expires_at` before today |
| `L004` | `tolerance_override` is at or below `tolerance_floor_at_creation` |
| `L005` | a decision's latest effective tier is below its set's current floor |
| `L006` | an acceptance actor, or an escape's `accepted_by`, is refused by §3.2 |
| `L007` | a stored `hash` does not equal the recomputed canonical hash |
| `L008` | an acceptance's `(decision, version)` pair matches no filed version |
| `L009` | an acceptance's actor is not the author of the commit that introduced it |

Notes that are part of the specification, not implementation detail:

- **Only the latest version of a decision is judged** by `L001`, `L003` and
  `L005`. An acceptance of a superseded version was already invalidated when
  the hash moved; reporting it again is noise on a resolved fact.
- **A revoked acceptance is not judged** for expiry.
- **`L008` checks the pair.** An acceptance naming one decision while signing
  another's hash is signing nothing about the decision it claims to accept.
- **`L009` skips, never fails, when there is no introducing commit.** An
  uncommitted acceptance is the state every acceptance passes through; failing
  it would make an acceptance impossible to commit in the first place. The
  check lands on the next run over committed history, which in practice is
  CI. A skipped check is always reported — a silently unrun rule reads as a
  passing one.
- **Allocated-awaiting-acceptance is status, not a failure.** The gate
  polices violations, not pendency. A gate that fires on ordinary work in
  progress is a gate people learn to ignore.

### 5.3 Gates and exit codes

`--gate readiness` blocks produce and runs every class except `L002` and
`L003`, which are dispositions that must hold at release rather than
preconditions for starting. `--gate completeness` blocks release and runs
all nine. With no flag, all nine run.

| Exit | Meaning |
|---|---|
| `0` | conformant |
| `1` | findings |
| `2` | the gate could not run (no store, unreadable file, bad flag) |

CI has to tell "the gate said no" apart from "the gate broke". This differs
from `ddd validate`, which returns `1` for both.

---

## 6. Open edges

Stated so an adopter meets them in this document rather than in production.

- **`L009` cannot see uncommitted work**, by construction (§5.2). A repository
  with no git history has the check skipped entirely.
- **`L006` does not cover a `judgment`'s `actor`**, nor `actor:` discharge
  pointers. §4.4 of the PRD scopes the rule to `accepted-by`, and the nine
  classes are closed. A judgment allocated to a model is therefore
  representable. This is a real gap and a candidate for L1.
- **The model-identity list is a floor** (§3.2), not a proof.
- **`constraint` carries no discharge requirement.** §4.4 does not impose one,
  so neither does this format — a constraint with no encoder is possible and
  is not a finding.
- **`expires_at` is optional** pending OD-6. An acceptance without one never
  goes stale, which is precisely the risk OD-6 has to settle.
- **A class-scoped acceptance still signs a version hash.** The scope widens
  what the acceptance covers; it does not loosen what it names. L1 owns the
  operational semantics of `accept --class`.
- **Nothing verifies the set.** Coverage is measured against the enumerated
  set (PRD §8), and enumeration completeness has no mechanical check at any
  milestone. Late-discovery rate is the lagging proxy.
- **ULID generation is not specified here** because L0 mints no ids. L1's
  `add` needs it.

---

## 7. What L1 needs from L0

The authoring operations reuse these library surfaces rather than growing a
second copy of the rules:

| L1 operation | Calls |
|---|---|
| `add` | ULID generation (new), `DecisionSet::validate_id`, `version_hash` |
| `allocate` / `escape` | `Allocation::assemble` (the §3.6 obligations, including `L002`) |
| any write | `Tolerance::new` (the `L004` rejection at write) |
| `accept` | `Identity` parse plus `model_or_bot_reason` (`L006`), `version_hash` to sign |
| `revoke` | the acceptance/revocation schemas |
| `status` | `verify::view::View` — latest versions, live acceptances, pendency |
| `blame` | `blame::introducing_author` |
| `diff` | `canon::canonical_json`, field by field |
