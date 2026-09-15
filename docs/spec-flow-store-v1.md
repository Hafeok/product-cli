# The specification-flow store — format v1

**Status:** normative. The code follows this document, not the other way round.
Companion to `docs/ledger-format-v1.md`, which it borrows its hashing law and
its closed-class discipline from.

**What this covers.** Every file the specification flow writes — act-time
records, ratified acts, filed refusals — the closed set of verdict classes the
store can fail, and the rules that keep a machine from writing the ones that
name a principal.

**Why it is a file and not a suspended process.** `implement` can be run by an
agent; `close` cannot. Those two acts are separated by an unbounded amount of
wall-clock time and, usually, by a process boundary — the agent run ends, a
person closes the record the next morning. A durable record on disk is
therefore the only representation that survives the gap. An in-process
continuation, a workflow checkpoint, or a held transaction would all tie the
human's decision to the lifetime of the machine's run, which is exactly the
coupling §5 of the specification-flow PRD exists to break.

---

## 1. Store layout

```
.spec/
  inventory.json      # the importer's projection — derived, never authority
  records/<ulid>.yml  # one act-time record
  acts/<slug>.yml     # one ratified act
  rejections/<slug>.yml   # one filed refusal
```

One subject per file, named by its own id. Nothing is edited after it is
written; a correction is a new file, the same rule the ledger's log files
carry.

**`inventory.json` is a projection, not a record.** It is rebuilt wholesale by
every `import`, carries no verdict, no ratification and no decision, and is
written by the agent host rather than by `spec`. Deleting it loses nothing a
re-run does not restore. Everything else under `.spec/` is authority, is
written only by `spec`, and names a principal where one is owed.

## 2. The open record

```yaml
form: spec.act-record.v1
id: 01K5CJ7Q3S8XN2VYB4M6E9TZRA
act: implement
slice: checkout-totals
act_ref: act/settle-basket
opened_at: 2026-09-15T09:14:02Z
opened_by: agent@example.invalid
base_revision: 9f3c1d0e5a7b2c4d6e8f0a1b3c5d7e9f1a2b3c4d
binds: sha256:…
```

| Field | Meaning |
|---|---|
| `form` | the format discriminator; a change to the hashed field set bumps it to `.v2` with a migration note |
| `id` | ULID, minted at open |
| `act` | the flow verb that opened the record. `implement` is the only one in v1 |
| `slice` | the slice built against the specification |
| `act_ref` | the specification act the slice realises |
| `opened_at` | RFC3339, UTC |
| `opened_by` | the opener's identity. **May be a machine** — building is delegable |
| `base_revision` | the commit the work started from |
| `binds` | the record's own opening hash (§4) |

## 3. The closure

A closed record carries one additional key. Its absence is what `S001` fails on.

```yaml
closure:
  kind: determinations
  principal: emil@example.com
  at: 2026-09-16T08:30:11Z
  determinations:
    - det/basket-rounding-is-half-even
  binds: sha256:…
```

`kind` is one of exactly two values, and silence is neither:

| `kind` | Requires | Means |
|---|---|---|
| `determinations` | a non-empty `determinations` list | determinations were produced while acting, filed at the address they were read from |
| `nothing-arose` | an empty or absent `determinations` list | a positive declaration that acting produced no determination |

`nothing-arose` is a declaration, not an absence — the same shape as
`asserted-none` for an uncovered set. A record with no `closure` key is not
`nothing-arose`; it is open, and it fails the gate.

## 4. Hashing

The `binds` fields are domain-separated SHA-256 digests over the canonical JSON
of the fields they cover, computed through `ledger_core::hash::domain_hash` —
one hashing law in this workspace, never a second scheme.

| Digest | Prefix | Covers |
|---|---|---|
| record | `spec.act-record.v1` | `act`, `id`, `act_ref`, `base_revision`, `opened_at`, `opened_by`, `slice` |
| closure | `spec.act-closure.v1` | the record digest, plus `at`, `determinations`, `kind`, `principal` |

The closure digest covers the record digest, so a closure names the exact
opening it discharges. Editing an opened record after it was closed breaks the
binding and fails `S003` — the same mechanism as a seam binding naming its
transition, applied to the write-back.

## 4a. Acts and refusals

A **ratified act** (`acts/<slug>.yml`) is what a principal said the codebase
does. It carries `name` and `settles` — the two things a tick cannot supply —
plus `realised_at`, the entry points it is realised at. Several is a legitimate
answer: transport boundaries are often finer than act boundaries.

A **filed refusal** (`rejections/<slug>.yml`) is a candidate a principal looked
at and declined, with the reason. Filed rather than discarded, because a
refusal is a decision and a flow that forgets its refusals re-asks the same
question at every import.

Both bind their content the way a closure binds its opening — `spec.act.v1`
and `spec.rejection.v1`, same law, own prefixes — so editing a ratified act
after the fact breaks its binding and fails `S007`.

**A refusal covers an entry point.** `S005` fails on silence, not on the
absence of an act: a principal who looked at an endpoint and said *this is not
an act* has made exactly the decision the class exists to require.

## 5. Verdict classes

The set is **closed**. The store fails for a schema fault plus these seven
classes and nothing else; adding an eighth is a change to this document, not a
patch.

| Class | Fails when |
|---|---|
| `S001` | a record carries no `closure` — the write did not happen |
| `S002` | a `principal` resolves to a model or a CI identity — on a closure, a ratification or a refusal |
| `S003` | a closure's `binds` does not match the record it claims to close |
| `S004` | a closure declares `determinations` with an empty list, or `nothing-arose` with a non-empty one |
| `S005` | an entry point no ratified act covers, and no principal has refused |
| `S006` | a record names an act that is not ratified |
| `S007` | a ratification or refusal whose `binds` does not match its content |

All seven are **structural**. None is configurable by project policy: a project
that could switch `S001` off would have a tool that reports what it was told to
report.

**One judgement to flag.** The PRD is not consistent about drift. §6 puts *an
entry point with no accepted act* in the fail list and calls it "the one worth
a build failure"; §7's structural table omits it, and §12 shows the
seam-scoped `unmapped_entry_points_on_seam` as a *policy* verdict with a basis.
This format follows §6 and makes `S005` structural, on the reading that §12's
metric is a narrower, seam-scoped gate a project may add on top. If the
intended reading was the other way round — drift reported, gated only by
policy — this is the line to change, and it is one line.

`S006` is not judged until at least one act is ratified. A repo mid-adoption
is not a broken one, and failing every record in it would make the first
`implement` impossible to run.

## 5a. What reports rather than fails

`map` produces a restructuring work list, and none of it is a verdict:

| Finding | Meaning |
|---|---|
| **merge** | several entry points, one act — transport boundaries finer than act boundaries |
| **split** | one entry point, several acts — an entry point spanning an act boundary |
| **unmapped entry point** | drift, or an act nobody has named |
| **unmapped act** | specified and unrealised |

The metrics beneath the verdicts — entry points, candidates, unreviewed
candidates, mapping coverage — are likewise reported and never gated. A
codebase with unspecified regions is unspecified, not non-conformant. A
codebase with dangling references is broken.

Coverage with nothing to cover reads `null`, not 100%: an empty codebase is
not fully specified, and a figure that says otherwise is the kind of
flattering default that makes a metric useless.

`S002` is the accountability leg made mechanical, and it applies wherever a
principal is named — closing a record, ratifying a candidate, refusing one. It
delegates to
`ledger_core::identity::Identity::model_or_bot_reason`, which is the same test
the ledger's `L006` applies to an acceptor — one identity law, two gates. That
test is a floor and not a proof: it catches the identities a CI system or an
agent harness produces by default, which is where the failure actually occurs,
and it cannot catch a model configured with a human-looking address.

**What v1 does not claim.** A closure is not cryptographically signed. `S002`
establishes that the named principal does not *look* like a machine; it does
not establish that the named human closed it. The ledger reserves
`Acceptance.signature` for that upgrade and leaves it empty at L0, and this
format inherits the same honest limit. Any statement that a machine "cannot
forge a closure because it does not hold the key" is false until that field is
populated on both sides.

## 6. Exit codes

| Code | Meaning |
|---|---|
| `0` | conformant |
| `1` | findings |
| `2` | could not run |
| `3` | work completed, closure pending |

`3` is what `implement` returns when it opened a record and did not close it —
which is every unattended run. It is not success. It is also not failure, and
keeping it distinct from `1` is what lets an agent harness tell "I finished my
half" apart from "I broke something".
