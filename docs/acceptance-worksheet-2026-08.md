# Acceptance worksheet — 2026-08

**Purpose:** make the pass over the pending ledger entries a reviewable
morning rather than a shell loop.

**What this document is not:** the entries themselves. `ledger show` is the
read — one screen per decision, carrying the hashed content, both edge kinds
kept apart, each edge's *argument* under its pointer, the filing act and the
acceptance situation. This worksheet is only the grouping, the flags, and
the sequence. Where the two disagree, `ledger show` is right: it is derived
from the log, and this file is written by hand.

**State when written** (2026-08-13, after the `revisit_if` amendment):

| | count |
|---|---|
| awaiting acceptance | 79 |
| escaped, priced | 1 |
| **pending total** | **80** |
| already decided (`ledger-design`) | 12 |

The count was 79 in the session that scheduled this pass; it is 80 because
that session filed one new decision — the masked-gate remedy — which is in
group B below and flagged there.

---

## The two groups

The split is **derived, not hand-listed**: `ledger show --group` computes it
from the discharge, so it cannot go stale as entries are added. A decision
is *mechanical* when it is a criterion discharged solely by the
repository-diff contract check; everything else is an individual read.

### Group A — 33 transcribed seam declarations (mechanical)

```
ledger show --group mechanical
```

**Read the group, not the entries.** All 33 are one shape, and that is
checked rather than asserted — every one of them is:

- `allocation: criterion`, `discharge_stage: pr`;
- discharged by exactly one `contract:seam/…` pointer;
- grounded by exactly one `ddd-content:seam/…` basis and nothing else;
- no reopen edge, floor `T1`, first version, all filed 2026-08-12 by the
  M8 migration.

By adapter: 19 `seam/web/…`, 10 `seam/ledger/…`, 3 `seam/rust/…`,
1 `seam/mcp/…`.

**What you are actually ruling**, once, for the group: *that the
transcription is faithful* — that each ledger entry says what its `.ddd`
seam declaration says, and that the contract check named as its discharge is
the right checker for it. You are not re-ruling the seam declarations; those
were decided when they were declared. If the transcription is faithful, the
group is one judgment.

**What would break the group** and force entry-by-entry reading: any entry
whose statement does not match its seam's `contract_location`, or whose
`ddd-content:` pin no longer resolves. Neither is currently reported as
drifting — `ddd report escapes` shows the ledger section clean apart from
the two basis-loss rows and the one reopen row, none of them in this group.

### Group B — 47 individual reads

```
ledger show --group individual
```

46 awaiting plus the one priced escape. Each is a separate judgment; the
list below is one line each on *what you are ruling*, not on what the entry
says. Read them from `ledger show`, which now prints each basis's argument
text under its pointer — for the re-typed entries that argument is the
thing being ruled on, so it belongs on the screen, not in this file.

The 12 `ledger-design` decisions are already decided and are not in this
pass.

**The sub-groups inside B.** They partition the 47 — every entry is in
exactly one — and are listed in the order worth reading them:

1. **The re-typed bases — 7** (the 2026-08 basis-quality audit's seven):
   `git-is-the-amend-trail`, `internal-not-surface`, `no-unwrap`,
   `predicates-carry-no-status`, `what-boundaries-priced-not-paid`,
   `m6-proceeds-no-flip`, `rust-class-enforced-here`. Ruling: that the
   mandate/constraint/preference now stated as the ground *is* the ground,
   and that the claim demoted from it was never doing the work. Both
   batches were ratified in `.ddd` on 2026-08-12; what is pending here is
   the ledger entry recording them.

   **Three of these seven carry a second ruling** and are version 2 of
   their decision: `internal-not-surface`, `no-unwrap` and
   `m6-proceeds-no-flip` are the `revisit_if` conversions of 2026-08-13.
   For those three you are also ruling that the edge belongs on the reopen
   side rather than the ground side, and that carrying its pin across
   unchanged was right. Version 1 said the same thing with the edge as a
   `watched:` marker inside `based_on`, and is now history. One screen
   covers both rulings; `revisit if` is its own block on it.

2. **The ruling record — 1.** `question/ddd/watched-edge-kind`, version 2.
   Ruling: that the statement now on file is the ruling you gave.

3. **The derived allocations — 37.** The M8 migration read each `.ddd`
   decision's own shape and allocated accordingly — mostly `judgment` with
   you as actor, four `constraint`. Ruling: that the allocation the
   migration derived is the one you would have chosen. The largest
   sub-group, and the one where a wrong derivation is easiest to miss,
   because a judgment allocated to you reads as unremarkable. One of the 37
   is the held entry (`interceptor-not-extension`), so 36 are acceptable.

4. **The risk record — 1** (`escaped-priced`).
   `risk/ddd/undeclared-what-boundaries`
   (`dec:hafeok.ddd/01KZTGGX5ABSQ2PVTQ32NPKVNE`) — 24 undeclared What
   boundaries, exposure accepted until review. Ruling: that the exposure is still worth
   carrying, and that the review date still holds. The only entry in the
   whole pass where accepting means accepting a *known* cost.

5. **The masked-gate remedy — 1** (new, 2026-08-13).
   `ci-gates-report-skipped`. Ruling: that a red build should report which
   gates did not execute, and that the judgment allocation is right while
   nothing checks it. Filed for acceptance; not implemented.

---

## Must not be accepted yet

**`dec/ddd/interceptor-not-extension`** — `dec:hafeok.ddd/01KZTGGKYN3VT9XYPATK0S0SAA`.
Its basis is `indeterminate:DDD-arch-03`: the edge is filed as
*indeterminate*, awaiting your F-7h ruling. Accepting it signs that
indeterminacy into the record as though it were settled. Rule F-7h first;
the acceptance is a separate act afterwards, against whatever version that
ruling produces.

**Nothing else is held.** In particular, nothing is mid-migration: the three
`revisit_if` conversions all landed as complete versions with both stores
gates-green, and the fourth entry the amendment touched (the ruling record)
likewise. If a conversion had been left half-filed it would appear here; it
does not.

One thing is *blocked* rather than held, and it is not in this pass at all:
the provenance audit's one upstream watched-not-grounding row
(`workspace-member-delivery` → the What/How vocabulary) is still unfiled,
waiting on a cross-repo reference shape that has not landed. There is
nothing to accept for it because there is nothing filed.

---

## The uniform-T1 observation

**Every one of the 80 sits at `T1`, the `ddd-governance` floor. Not one
carries a `tolerance_override`.** The tolerance floor therefore did no
discriminating work in this migration: it was pinned, not chosen. That is
expected of a mechanical migration — it carries forward what was declared,
and no tier was declared — but it means the tier tells you nothing about
which entries deserve more of your attention, and this worksheet's grouping
is doing the job the tier would otherwise do.

**Proposed as a genuine T2 case (a proposal for `revise`, not applied):**

- **`risk/ddd/undeclared-what-boundaries`** and its decision
  **`dec/ddd/what-boundaries-priced-not-paid`**. These are the pair that
  knowingly carries exposure across 24 boundaries with a review date. T2 is
  where a signature is meant to cost more, and a priced escape is the one
  shape in this store where "who signed this, and how carefully" is the
  whole question. It is also the pair that L6's certificate signing would
  bind first, since that revision requires a signature only above the floor
  — at a uniform T1 it would require none anywhere.

**Considered and not proposed:** `dec/ddd/typed-basis`. It reshapes every
future entry, which is an argument for weight, but it is a *format* ruling
whose consequences are mechanically checked by `ddd validate`; the check,
not the tier, is what holds it. Raising it would be tier inflation.

**Cost of acting on the proposal**, stated so it is not a surprise: an
up-only override goes on a *new version*, and `ledger revise` has no
`--tolerance-override` flag today — only `ledger add` does. So applying it
is a small CLI addition plus two revisions, and each revision returns its
decision to awaiting-acceptance. Raising the *set floor* instead is the
wrong instrument: it strands every member pinned below it (`L005`) until
each is re-pinned.

---

## The sequence

Run from the repo root, on a branch, with the store clean
(`ledger verify` exit 0 before you start).

**0 — identity.** `ledger accept` takes the acceptor from git config, and
`L009` checks that the acceptance's actor is the author of the commit that
introduced it. So both must be you:

```sh
git config user.email          # must be emk@delegate.dk, not a model or CI identity
git config user.name
```

If the commit ends up authored by anyone else, `ledger verify` fails
`L009`. `--no-blame` skips that check; it is a bypass, not a fix, and using
it defeats the one mechanism that catches an acceptance filed under a
borrowed identity.

**1 — group A, one read then one recorded judgment.**

```sh
ledger show --group mechanical | less     # the read; this is the actual work
```

Then, having read it, record the one judgment across the group. Two
invocations, not a shell loop:

```sh
ledger accept --group mechanical                      # enumerates, writes nothing
ledger accept --group mechanical --confirm sha256:…   # the manifest the first printed
```

The first prints the selection — every member's id, hash, weight class and
state — together with a **manifest**: one digest over exactly those
`(decision, hash, standing)` triples. It writes nothing. The second signs,
and refuses unless the store still enumerates to that same manifest: if
anything was filed in between, the run stops and names what moved rather
than quietly signing an entry you never read. The confirm value cannot be
typed by accident, because only the first invocation can produce it.

**What is batched, honestly:** the *act*, never the signature. The format has
no group-acceptance primitive and this verb does not invent one — an
acceptance signs one version hash, and `scope: class:<ref>` parses but still
signs one version. `--group` files 33 acceptance records, each signing its
own hash, each through the same gate a single `ledger accept` runs. What it
removes is 33 invocations, not 32 judgments: the *reading* still happens
once, above, for a group proven to be one shape. If reading the group left
you unsure about any single entry, pull it out and read it on its own screen
first — a batch is not the place to resolve a doubt.

The verb is the CLI standing in for the acceptance workbench (L5), the same
way the PR-review surface stands in for it elsewhere. It is an interim
affordance, recorded as one.

**2 — group B, entry by entry.**

```sh
ledger show --group individual | less     # or one at a time:
ledger show dec:hafeok.ddd/<ulid>
ledger accept dec:hafeok.ddd/<ulid>
```

`--group individual` is deliberately **not** batchable while the held entry
stands: `ledger accept --group individual` refuses the whole run and names
`dec:hafeok.ddd/01KZTGGKYN3VT9XYPATK0S0SAA` (`interceptor-not-extension`),
because a batch that signed around it would make that judgment silently.
That refusal is derived from the entry's own `indeterminate:` basis, not
from this file, so it cannot go stale. Group B is 47 separate judgments
anyway; the pressure to rule F-7h first is the mechanism working.

**3 — check and commit.**

```sh
ledger verify                # exit 0; L009 will police step 0
ledger status | tail -20     # should show 1 awaiting (the held entry)
ledger reindex               # the L2 index follows the log
git add .decisions && git commit -m "Accept the 2026-08 pending batch"
```

---

## Wall-clock

Record the elapsed time of the pass here when it is done. It is not
bookkeeping: this is the **first real reading of the acceptance-latency
instrument** — how long a principal actually takes per entry, split by
weight class — and it is the requirements input for L5, the acceptance
workbench. A workbench designed without it would be designed against a
guess.

`ledger accept` reports its own `elapsed` on every invocation, refusals
included, so the two grouped runs supply their half directly; the reading
time either side of them is the part only you can record.

Capture at least: total elapsed, and the split between group A (one read
plus the two invocations) and group B (per-entry). If group A's per-entry cost turns
out to be near group B's, the grouping did not buy anything and L5 should
know that too — a null result here is as useful as a positive one.

| | start | end | elapsed | entries | per entry |
|---|---|---|---|---|---|
| Group A (mechanical, 33) | | | | 33 | |
| Group B (individual, 47) | | | | 46 accepted, 1 held | |
| **Total** | | | | **79 of 80** | |

Notes on anything that slowed the pass down (an entry that needed the `.ddd`
file opened, an argument that did not render, a screen that lacked something
you wanted):
