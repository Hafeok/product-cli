# Session bootstrap — C-track, session 1

**Convention:** `DDD-dec-20`, two-file form. This is the second file; the first is the committed
session prompt (path below). Filed late, at Gate 2b, not at Gate 1. Recorded as late rather than
backdated.

---

## Principal

| | |
|---|---|
| Principal | Emil |
| Ratification | by merge; the session proposes at gates and does not self-ratify |
| Session agent | Claude Code |

## Branch and bases

| Repository | Role | Base | Branch |
|---|---|---|---|
| `Hafeok/product-cli` | working | `d0f4297` (`origin/main`) | `claude/c-track-factoring-gate-claim-kmk0hh` |
| `Hafeok/decision-driven-design` | read-only reference | `efb46682251c74d8396ecf90518a38d1c711eab7` | — |
| `Hafeok/actor-indexed-determination` | not touched | — | — |

**Two corrections to the supplied draft, both about the upstream row.**

1. The draft records the upstream repository as *"fetched at Gate 1"*. It was not. It was fetched
   during the **Gate 2** turn, when the scope of `DDD-dec-20` was checked. At Gate 1 the decision had
   not been read and its scope was recorded as unresolved. Corrected rather than carried.
2. The draft anticipated that the base could not be recovered. It could. The clone was taken during
   this session and is still on disk, so the commit the decision was read at is recorded above as
   fact, not reconstructed.

**A limit on that base, stated rather than left implied.** The clone is `--depth 1`. `efb4668` is the
repository head at fetch time, which is what the decision was read at — but a shallow clone carries
no history, so this session **cannot** state which commit last modified
`core/decisions/DDD-dec-20.yaml`. The pinned ref is the read ref, not the authoring ref, and those
are different facts.

## Charter

| | |
|---|---|
| Gates | 5 as chartered (Gate 0 live state → Gate 5 session close), plus **Gate 2b**, interposed by the principal's Gate 2 reply |
| Milestones in scope | C0 factoring gate; C1 scaffolding only if C0 ratified in-session |
| Track | C (claim register core) |

**Gate 2b is not in the committed prompt.** It was added by the principal after Gate 2 and is
recorded here because a gate the charter does not contain is exactly the divergence between
charter and conduct that this file exists to make visible.

## Prompt artefact identity

**As supplied, before any in-session amendment:**

| | |
|---|---|
| Lines | 142 |
| sha256 | `8139988fd57744a15fab715b4227065719f095a6b5666f6ba946fe81c27af722` |

**As committed:**

| | |
|---|---|
| Path | `docs/c-track/c0-session-prompt-2026-08-26.md` |
| Lines | 172 |
| sha256 | `6c2ae40b1ae5796aa436616b14f5c8390ffb3be8f2532e478ef1c6b8dc34659f` |

The two differ, as the draft predicted: the committed file is the supplied prompt plus the Gate 1
note on filing location and the Gate 2 correction appended to it.

**The divergence is accounted for exactly, not approximately.** The committed hash was computed from
the committed bytes, never copied. Independently:

```
$ head -142 docs/c-track/c0-session-prompt-2026-08-26.md | sha256sum
8139988fd57744a15fab715b4227065719f095a6b5666f6ba946fe81c27af722
```

The first 142 lines of the committed file hash **bit-for-bit to the supplied hash**. The prompt was
therefore transcribed verbatim — no silent normalisation of whitespace, quotes or line endings — and
the whole of the difference between the two hashes is the 30 appended lines. That is the strongest
form the identity record can take: not "the difference is explained" but "the difference is the
appendix and nothing else".

**`[OPEN]` Path divergence from the draft.** The draft anticipated `docs/c-track/prompt.md`; the
committed path carries the `c0-` prefix and the date, following the `docs/g-track/` naming this
repository already uses. That breaks the flat `prompt.md` / `bootstrap.md` pairing `DDD-dec-20`
describes. **Not renamed**: the file is committed and hashed, and quietly renaming a hash-identified
artefact to match an expectation is the tidying the identity check exists to catch. Recorded as
divergence for the principal to rule on; if renamed, both hashes stay valid and only the path row
changes.

## Basis

- `prd-claim-register-core-2026-08-26.md` — the C-track PRD, `[PROPOSED]`, not in this repository
- `prd-decision-to-behaviour-claim-register-2026-08-26-r2.md` — not in any repository reachable here
- `annex-a-corpus-acquisition-2026-08-26.md` — not in any repository reachable here
- sibling ruling `DDD-dec-NN` — drafted, unnumbered, unmerged
- `DDD-dec-20` — upstream, out of scope for this session by its own scope clause; followed
  voluntarily. Read at `efb4668`.

`[OPEN]` Four of the five basis documents are outside every repository reachable from this session.
Where the session worked "per r2 §7.1", it worked from the C-track PRD's restatement, not the source.
That is a basis-loss condition and it is recorded here rather than discovered later.

**Gate 2b sharpens this from a disclosure into a blocker for one deliverable.** The provenance audit
(§3.1 of the principal's reply) asks each element to be traced *to r2 §7.1 as filed*. r2 is not
reachable, so what the audit can produce is an audit of **attributed** provenance — what the C-track
PRD says about each field's origin — not of **verified** provenance. The distinction is carried on
every row of `c0-provenance-audit-2026-08-26.md` and is the audit's principal limitation.

## Scope note

`DDD-dec-20` binds sessions in `decision-driven-design` and cross-repository sessions from a
downstream branch. This session touches neither repository and falls outside its scope. The
convention is followed by choice. No upstream deviation arises and none is filed.

## Automation

None. `DDD-dec-20` is deliberately unautomated — no validator checks that a session committed its
prompt, and none is proposed here either.
