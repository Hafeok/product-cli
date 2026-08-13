# Decision Ledger — Entry Format v1

**Status:** normative for `format: 1` through `format: 4`.
Specification revision **v1.5** (2026-08-13): introduces `format: 4`,
which adds one optional version field, `revisit_if` — the reopen edge,
ruled by the principal a **distinct edge type and never a basis** (§3.7).
The field is hashed when present and omitted when absent, so **every
existing digest is unchanged** and `CANONICAL_FORM` does not bump; the
file gate's ten classes are unchanged. Revision
**v1.4** (2026-08-12, ddd M8): introduces
`format: 3`, which adds one discharge scheme, `contract:` — the
repository-diff contract check as a discharge kind (§3.4). No field
changes, no hashed-meaning changes: **every existing digest is
unchanged** and `CANONICAL_FORM` does not bump. Revision **v1.3**
(2026-08-11, L3) introduced `format: 2`, which adds one
optional version field, `merged_from` — the other tip a merge arbitration
closed. The field is hashed when present and omitted when absent, so
**every format-1 digest is unchanged** and `CANONICAL_FORM` does not bump;
the graph stage gains `G005` (competing supersession). Revision **v1.2**
(2026-08-11) defined "latest" as derived from the version parent DAG
rather than file or ULID order (§5.2) and added `G004`, the forked-chain
shape (§8). Revision v1.1 (2026-08-10) added gate class `L010`. The file
gate's ten classes are unchanged by all four revisions — see
`ledger-format-migrations.md`.
**Scope:** L0 of `decision-ledger-prd.md` — the file format, the canonical
form, the version hash, and the `verify` gate. No graph, no index, no merge,
no coverage query, no federation. (The L2 graph stage reports through the
same `verify` command but is a distinct stage outside this document's
class set; see §8.)
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
| `contract` | a declared boundary (seam id or `file#symbol`) | `contract:seam/ledger/verify-classes` |

An unknown scheme is a schema fault: a pointer nothing can ever resolve is
prose, and prose is what this format exists to replace. Nothing *resolves*
these at L0.

`contract` is the **format 3** scheme (spec v1.4, added through this
document's amendment procedure for the ddd M8 integration): the decision
is discharged by the repository-diff contract check — a change to the
named boundary in any revision range must carry a declaration signing
that exact change, validated in CI by the shared classifier. A file
carrying a `contract:` pointer declares `format: 3`; a lower-format file
carrying one is a schema fault, and a store that never uses the scheme
stays a pure format-1/2 store. Hashing is unaffected: a discharge pointer
was always hashed by its string form.

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
    merged_from: sha256:...             # format 2 only: the tip a merge closed
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
    revisit_if: [claim:DDD-adapter-02@sha256:…]   # format 4 only; not ground
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
is deliberately unspecified. That upgrade is now scheduled — spec v1.6 /
`format: 5`, PRD §4.5 / milestone L6 (renumbered a third time: its
original `format: 2` slot was consumed by L3's `merged_from`, its
`format: 3` slot by the M8 `contract:` scheme, and its `format: 4` slot by
the v1.5 `revisit_if` edge; nothing else about the plan changes) — and
remains **planned, not normative**: under every
format this document specifies, a non-empty `signature` is still a
schema fault (§6, last bullet).

`merged_from` (spec v1.3) exists only at `format: 2` — a format-1 file
carrying it is a schema fault. It names the *other tip* a merge
arbitration closed: a reconciled version extends its `parent` chain and
closes the divergent chain it settled against, which is how a fork
(`G004`) heals inside the DAG. A file declares the format it actually
needs, so a store that never merged remains a pure format-1 store. The
reconciled version is a new version: no acceptance survives reconciliation
— prior acceptances keep signing exactly the historical versions they
named, and the reconciled content awaits a fresh signature.

`based_on` is a list of single-token basis pointers. Its vocabulary is
**open** at L0 — nothing dereferences a basis pointer yet, and closing the
vocabulary now would reject adopters' existing reference schemes for no gain.
§9.4 of the PRD closes it at L4.

`revisit_if` is specified in §3.7. It is **not** part of `based_on` and
never appears inside it.

### 3.7 The reopen edge — `revisit_if` (format 4)

Ruled by the principal (2026-08-13), settling the watched-edge question the
2026-08 basis-quality re-typing session left open and the ddd M8 migration
carried as a provisional `watched:` marker *inside* `based_on`.

A `revisit_if` pointer names a claim whose **death reopens the decision**.
That is the converse of ground, not a weaker form of it: the decision does
not rest on the claim, so falsifying the claim does not undermine the
decision — it obliges someone to look at it again.

```yaml
format: 4
versions:
  - decision: dec:hafeok.ddd/01KZ…
    based_on: [mandate:dec/ddd/internal-not-surface]
    revisit_if: [claim:DDD-adapter-02@sha256:b333063d…]
```

Rules:

- **A reopen edge is never a basis.** It lives in its own field with its
  own vocabulary. Writing one inside `based_on` — as a `watched:` token or
  under any other marker — is not the way to say this, and a consumer must
  not read `revisit_if` as ground. In the reference implementation the two
  are distinct *types* (`RevisitRef`, `BasisRef`), so the separation is not
  a convention anyone can forget.
- **The two report differently.** A claim on a `based_on` edge moving
  produces a **basis-loss** finding — the ground shifted under a standing
  decision. A claim on a `revisit_if` edge moving produces a **reopen**
  finding — the tripwire fired and the decision is due a fresh look. These
  are different facts about a decision and a report that merges them tells
  the reader neither. Neither finding is a gate class (see below).
- **Same declare-what-you-need rule as formats 2 and 3.** A change-set
  declares `format: 4` only when one of its versions actually carries a
  `revisit_if`; a store that never states one stays a pure format-1/2/3
  store, and a lower-format file carrying the field is a schema fault.
- **The vocabulary is open**, exactly as `based_on`'s is: L0 dereferences
  no pointer. Open is not shared — the pointer types stay distinct.
- **Hashing.** `revisit_if` joins the hashed field set as a list (a *set*,
  like `discharge` and `based_on`: deduplicated, code-point sorted,
  reordering is formatting). An absent key is omitted from the canonical
  object (§4.2 step 3), so every version written before the field existed
  canonicalises to byte-identical content: no digest moves, no acceptance
  is invalidated, and the prefix stays `ledger.decision-version.v1`. The
  two lists canonicalise under **separate keys**, so one token filed as
  ground and the same token filed as a reopen edge are different content —
  an acceptance always names which of the two it signed.
- **Not a gate class.** The file gate's ten classes are unchanged: nothing
  here fails `verify`. Resolving a reopen pointer — does the claim exist,
  has it moved — is a consumer's business at L0, the same posture every
  discharge scheme and every basis pointer already has. An eleventh class
  would be a further format-spec change, by the `L010` mechanism.

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
decision · parent · merged_from · set · statement · allocation ·
discharge · discharge_stage · actor · expectation · exposure ·
accepted_by · review_by · tolerance_floor_at_creation ·
tolerance_override · based_on · revisit_if · supersedes
```

`merged_from` joined the set at spec v1.3 (`format: 2`) and `revisit_if` at
spec v1.5 (`format: 4`). Because an absent key is omitted from the canonical
object (§4.2 step 3), every version written before either field existed
canonicalises to the same bytes as before: no digest moved, no acceptance
was invalidated, and `CANONICAL_FORM` stays `v1`.

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
4. **List fields are sets.** `discharge`, `based_on` and `revisit_if` are rendered as their
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

`ledger verify` fails for a **schema fault** or one of **ten classes**, and
for nothing else. An eleventh reason is a change to this document — `L010`
itself arrived that way, as the spec v1.1 amendment.

### 5.1 The parse gate

`SCHEMA` covers: a file that does not parse against the format it declares;
an unknown `format`; an unknown key; an unknown discharge scheme; a file stem
disagreeing with its declared id; a duplicate id; a per-allocation obligation
from §3.6 that is not met (except the escape's, which is `L002`); a
non-empty `signature`; a version naming an undeclared set; a revocation
naming an acceptance nobody filed.

### 5.2 The ten

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
| `L010` | a `judgment`'s `actor` is refused by §3.2 (spec v1.1) |

Notes that are part of the specification, not implementation detail:

- **Only the latest version of a decision is judged** by `L001`, `L003`,
  `L005` and `L010`. An acceptance of a superseded version was already
  invalidated when the hash moved; reporting it again is noise on a resolved
  fact.
- **"Latest" derives from the parent DAG, never from file or ULID order**
  (spec v1.2). The latest version of a decision is the unique version whose
  hash no other version of the same decision names as `parent`.
  Content-identical filings (one hash filed more than once) are one version.
  ULIDs order by one clock, and two writers' clocks prove nothing about
  parenthood — deriving latest from change-set order was the single-writer
  leak the L1 report named, retired here. A chain that cannot name one tip —
  two versions unclaimed as parents, the store two divergent writers leave
  behind — has no latest: an implementation must not resolve the ambiguity
  by any ordering heuristic. The reference implementation reports it as
  `G004` (§8) and refuses authoring verbs against the forked decision until
  a recorded arbitration (`ledger merge --resolve`) settles the chain.
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
all ten. With no flag, all ten run.

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
- **`actor:` discharge pointers are not covered by `L006` or `L010`.** The
  judgment-actor half of the gap v1.0 flagged here closed as `L010` in spec
  v1.1; a *discharge pointer* naming a model identity remains representable.
  A pointer is a reference to where discharge happens, not an allocation of
  accountability, so extending the rule there is a separate decision.
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
- **Planned, not yet normative — spec v1.6 / `format: 5` (PRD §4.5,
  milestone L6; ruled 2026-08-11, unimplemented; renumbered by the M8
  `contract:` scheme consuming `format: 3` and the v1.5 reopen edge
  consuming `format: 4`).** The signing revision:
  `signature` goes live as a detached, certificate-based signature (git's
  `gpg.format` trio — `openpgp` | `ssh` | `x509`) over a canonical
  acceptance payload — decision id, version hash, actor, signing timestamp
  — required only when the acceptance's effective tier is above the
  tolerance floor. The trust root (allowed signers) enters the store as
  governed entries with an explicitly-named genesis entry. Two classes will
  join the file gate by the same amendment mechanism that added `L010`:
  `L011` (a signature required at the acceptance's effective tier is absent
  or invalid against the trust root as of its signing timestamp) and `L012`
  (acceptances exist under a since-revoked key — a review trigger, since
  validity is judged at signing time, never retroactive invalidation) —
  taking the closed class count from **ten to twelve** when that revision
  lands. Hashing is unaffected: the signature is *over* the version hash,
  never inside it, so no digest moves and `CANONICAL_FORM` stays `v1`.
  Until that revision ships, everything in §§1–5 stands exactly as written.

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

L1 shipped against this table (2026-08-10); every verb routes through the
listed surface via a delta-gate — the verb refuses at write exactly the
findings `verify` would report afterwards. Semantic `diff <ref>..<ref>`
alone is deferred: it needs store-at-revision loading, which arrives with
L3's per-decision merge base.

---

## 8. The graph stage (L2)

**Outside the import surface.** Sections 1–5 are what an outside
implementation of the *format* reproduces; this section describes the
reference implementation's L2 graph stage, which an outside implementation
may skip without losing format conformance. The file gate's closed ten
classes (§5) are unchanged by it.

The `.decisions/index/` cache holds the RDF materialisation of the log
(`index/ledger.ttl`), rebuilt by `ledger reindex`. The log is the source of
truth; the emission is byte-deterministic, so deleting the index and
rebuilding reproduces it byte-identically — the PRD §5 correctness test,
run in CI. Acceptance provenance is PROV-O (an acceptance
`prov:wasAttributedTo` its actor; a version `prov:wasRevisionOf` its
parent; both `prov:wasGeneratedBy` their change-set). `based_on` tokens
become `ledger:basedOn` literals exactly as written — the vocabulary stays
open at this milestone; the graph exposes it and does not police it.

`ledger verify` additionally runs SPARQL shape checks over the emitted
graph — cross-entry referential integrity the per-file schema cannot
name. They report as a **distinct stage** of the same command, with
unchanged exit semantics (findings exit `1`):

| Code | Fails when |
|---|---|
| `G001` | a `supersedes` edge targets a decision no change-set filed |
| `G002` | a version's `parent` (or `merged_from`) hash matches no filed version of its decision |
| `G003` | a version names a decision no change-set introduced |
| `G004` | a decision's version chain forks into more than one tip (spec v1.2) |
| `G005` | one decision is superseded by two live claimants (spec v1.3) |

`G004` is the state two divergent writers leave behind — a plain git merge
of two branches' logs, each having revised the same decision from the same
parent. No file is malformed; the *store* cannot name a latest version, so
it is non-conformant until a recorded arbitration (`ledger merge
--resolve`) extends one chain past the fork, closing the other tip via
`merged_from`. A tip, for both `G004` and `G005`, is a version no other
version of the decision claims by `parent` *or* `merged_from`.

`G005` is the write-time one-superseder-per-decision refusal met across
branches, where it cannot refuse retroactively: each side's claim was
legal alone. Only live claims count — a claimant whose next version drops
the edge has withdrawn, which is exactly the arbitration act `ledger merge
--resolve` records for the losing side. The graph classes are closed the
same way the file classes are: a `G006` is a change to this section. Coverage (`ledger coverage`) reports the
seven-state disposition vocabulary — `undecided`, `awaiting-acceptance`,
`decided`, `escaped-priced`, `escape-review-due`, `expired`, `superseded`
— per set and per namespace, with supersession chains walked to their
tips, and always states §8-of-the-PRD's honest limit: coverage is measured
against the enumerated set, and nothing verifies the set itself.
