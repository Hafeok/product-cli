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
  policy/<ulid>.yml   # one version of the check policy, append-only
  trust/<id>.yml      # one trusted public key
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

## 4b. The check policy

`policy/<ulid>.yml` is a **determination, not a config file**. Versions are
append-only and a change is a supersession, so three things follow:

- *who lowered this, when, and on what basis* has an answer;
- the old threshold stays readable beside the argument that justified it — a
  policy loosened twice is evidence about the original claim, and editing one
  in place would destroy exactly that;
- project policy stays a determination rather than tool configuration.

**The version in force is the tip of the supersession chain**, derived from
`supersedes` and never from id order. A chain with two tips is forked: no
ordering heuristic may pick a side, so the gate says so rather than quietly
choosing one.

Every policy verdict declares three things, and the file is rejected if any is
missing:

| Field | |
|---|---|
| `fires_when` | the condition, mechanically evaluable |
| `basis` | why this threshold and not another |
| `principal` | who answers for it |

`principal.kind` is `human` or `team`. **There is no `machine` member**, and the
restriction is inherited from the type rather than applied by a check — which
is what makes it structural instead of instructed.

`fires_when` is deliberately tiny: `count <op> <number>` or `percent <op>
<number>`, with `op` one of `> >= < <= == !=`. A policy language rich enough to
be interesting is one rich enough to hide a threshold in, and the point of the
condition is that a reader can check it against the basis at a glance. A
percentage condition over a metric with no population never fires — firing on
an undefined figure would be a verdict about the absence of data.

`basis_binds` carries `basis_digest(fires_when)` and is **not** filled in on
the author's behalf. Moving a threshold must cost an edit at the basis; the
gate reports the digest to paste, the way `ledger verify` reports a version
hash for a hand-authored file.

**The default is structural only.** A project with no policy gets verdicts on
broken things and nothing else. Shipping default thresholds would presume a
basis nobody stated.

## 4c. Where the event model lives

The flow's `model` verb is **`product domain`**, not a second editor. The What
graph (§3.1/§3.2 of the framework) is already owned by `product-core`, and
re-implementing it here would give the repo two vocabularies for the same
thing.

This is also how §3 of the PRD is satisfied. *The domain model must be
authorable without candidates in view* is not enforced by a UI rule that
someone could relax; `product-core` does not depend on `spec-core`, so nothing
in the model path can read `.spec/inventory.json` at all. `map` is where the
two vocabularies meet for the first time, which is where the PRD puts them.
`spec-cli/tests/boundaries.rs` fails if that edge is ever added.

## 4d. What code may claim

Two attributes let code point back at the specification. Both are recognised
**by name**, so a project declares its own one-line attribute classes and takes
no package dependency on this tool — the importer has to run on codebases that
have never heard of it.

```csharp
[Slice("checkout-totals")]
public class Settlement
{
    [RealisesFact("det/basket-rounding-is-half-even")]
    public Money Round(Money m) => m;
}
```

| Attribute | Claims | Orphan when |
|---|---|---|
| `[Slice(id)]` | this code is part of a slice built against the spec | no act-time record declares that slice (`S012`) |
| `[RealisesFact(address)]` | this code realises a filed determination | no closure filed that determination (`S013`) |

An orphan is code asserting a link to a specification that does not exist. It
reads as governed and is not, which is worse than being plainly ungoverned.

> **Provenance of this section.** The specification-flow PRD names these two
> attributes in its fail list but does not define them; the document that would
> (`prd-csharp-stack-binding.md`) is not in this repo. The attribute names are
> the PRD's; the argument shape, the by-name recognition, and the reading of
> *fact* as *a determination some closure filed* are this format's, and are the
> lines to change if the missing document says otherwise. The PRD's other
> code-side conditions — the resolution conditions C-1…C-3 and the cross-scope
> seam disagreements CS-1…CS-4 — are **not** implemented, because they cannot
> be reconstructed from their names.

## 4e. Signing

`S002` establishes only that a named principal does not *look* like a machine.
A **signature** establishes that the holder of a key bound to that principal
actually acted. That is the difference between an accountability claim and an
accountability fact.

**Opt-in, at the repo.** Signing is off until `.spec/trust/` carries a key, and
turning it off means deleting trusted keys — a reviewable edit, not a flag on
one write. There is no per-record escape: once a key is trusted, every closure,
ratification and refusal needs one.

```bash
spec trust generate --id emil-2026 --principal emil@example.com --out ~/.keys/emil-2026.key
spec close <id> --principal emil@example.com --nothing-arose --key-file ~/.keys/emil-2026.key
```

The secret key is written outside the repo — a path inside it is refused,
because a secret key in a working tree is one commit from being public — with
owner-only permissions where the platform has them. `SPEC_SIGNING_KEY` carries
the same hex for CI-less environments.

| | |
|---|---|
| Algorithm | ed25519, hex-encoded, same as the `sha256:` digests |
| Signed subject | `spec.signature.v1` ‖ `
` ‖ the record's **recomputed** digest |
| Verified against | keys in `.spec/trust/` **bound to the principal the record names** |

Two properties fall out of signing the recomputed digest rather than the stored
one. Editing a signed record breaks its signature directly, so `S015` is a
guarantee on its own rather than one that leans on `S003` also being checked.
And because every digest already covers the principal it names, a signature
cannot be lifted onto another principal's record — there is no separate replay
guard to get wrong.

**Adopting later does not invalidate the past.** An act performed before the
repo trusted any key could not have been signed, so its *absence* of a
signature is graced. The grace is for absence only: a signature that is present
is verified whenever it was written. What pre-adoption records have instead is
`S003`/`S007` and the git history, and that is the limit rather than a claim.

**What this still does not establish.** That the human, rather than something
holding their key, acted. Key custody, rotation and revocation are not modelled
in v1: a compromised key is removed by deleting its file, which stops future
signatures and does not re-judge past ones.

## 5. Verdict classes

The set is **closed**. The store fails for a schema fault plus these fifteen
classes and nothing else; adding a sixteenth is a change to this document, not
a patch.

| Class | Fails when |
|---|---|
| `S001` | a record carries no `closure` — the write did not happen |
| `S002` | a `principal` resolves to a model or a CI identity — on a closure, a ratification or a refusal |
| `S003` | a closure's `binds` does not match the record it claims to close |
| `S004` | a closure declares `determinations` with an empty list, or `nothing-arose` with a non-empty one |
| `S005` | an entry point no ratified act covers, and no principal has refused |
| `S006` | a record names an act that is not ratified |
| `S007` | a ratification or refusal whose `binds` does not match its content |
| `S008` | a policy verdict that cannot be evaluated: a missing field, an unparsable condition, or a metric that does not exist (**B-1**) |
| `S009` | a basis that does not bind the threshold it justifies (**B-2**) |
| `S010` | one argument repeated — two verdicts in a policy carrying the same basis (**B-3**) |
| `S011` | a policy that lists nothing it deliberately does not gate |
| `S012` | an orphan `[Slice]` attribute — code naming a slice nothing declares |
| `S013` | an orphan `[RealisesFact]` attribute — code naming a determination no closure filed |
| `S014` | unsigned, in a repo that trusts keys (and not graced by §4e) |
| `S015` | a signature that verifies under no key trusted for the named principal |

All fifteen are **structural**. None is configurable by project policy: a project
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

**`S008`–`S011` are about the policy, not the project's thresholds.** A policy
whose basis does not bind is malformed, the same class as a dangling
reference; what the project chooses to gate is its own business. The two are
reported apart so a reader can always tell which is which.

**`S009` and `S010`, declared as a proxy with their divergence.** The proxy is
*basis present, bound to its threshold, not duplicated*. The original predicate
is *the threshold was reasoned rather than reached for*. The known divergence:
a padded, unique basis bound to its threshold passes — *because we said so, at
length* is admitted. Incidence unmeasured; no corpus exists. What `S009` does
catch on day one is the degeneration path exactly: quietly lower a number and
leave the argument that justified the old one.

`S011` admits an explicit `not_gated_asserted_none: true` — the same shape as
an `asserted-none` for an uncovered set. An *empty* `not_gated` with no
assertion is rejected, because a policy listing what fires without listing what
it deliberately does not is a coverage claim with no uncovered set.

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

**What v1 claims, and does not.** With a trust root, a closure *is*
cryptographically signed (§4e) and a machine cannot forge one without the key.
Without a trust root, `S002` establishes only that the named principal does not
look like a machine. Neither establishes that the human, rather than something
holding their key, acted — that is a custody question no file format answers.
The ledger still reserves `Acceptance.signature` and leaves it empty at L0, so
its acceptances remain unsigned; the two stores are not yet on the same footing
here.

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
