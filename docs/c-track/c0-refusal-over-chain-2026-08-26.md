# C0 — "one signature cannot discharge two boundaries", restated over a variable chain

**Status:** `[PROPOSED]` — session output, not ratified.
**Dated:** 2026-08-26. **Gate:** 2b §3.3. **Unblocks:** C-O-7, C-O-8.

The principal's hold: a corpus-declared chain of variable length makes this refusal a rule stated
over a structure with no fixed shape, and a refusal cannot be loose. Restatement first; ratification
of the variable chain follows it.

---

## 1. What the chain is

| | |
|---|---|
| **Shape** | An ordered list of named links, declared by the corpus. |
| **Length** | ≥ 1. Zero links is refused: a claim declaring no link records no transcription at all, and is not an empty statement but an absent one — the residue distinction (PRD §4.1) applied to the chain. |
| **Default** | r2/A-6's three: source→normalised, decision→specification, specification→implementation. A default, not a constant. |
| **Membership** | A link the corpus does not declare **does not exist for that corpus**. Corpus B declares two. |
| **Content** | Each link carries its own content hash, under its own domain-separation prefix — `ledger-core`'s canonicalisation law applied per link rather than per claim. |

That last row is the load-bearing one, and §2 is why.

## 2. The refusal

Stated so that the strong forms carry the weight and the checked form is only a backstop.

### R1 — structural: one acceptance binds one link

An acceptance record binds exactly one `(claim, link, identity)` triple. The type admits **one**
link, not a collection.

An acceptance naming two links is therefore not a value the refusal rejects — it is **not a
constructible value**. This is the FC-2 discipline applied at the type level: the adversarial test
"try to get round your own constructors" has nothing to reach for, because no constructor and no
builder accepts a second link.

### R2 — arithmetic: a signature is a digest of one link's bytes

An acceptance signs the **link's** hash, not the claim's.

A signature over link *i* cannot cover link *j*, because it is a digest over different bytes under a
different domain-separation prefix. Not "is rejected as covering" — **cannot cover**, in the same
sense that `ledger-core`'s acceptance-signs-a-hash makes a silent carry-over arithmetically
impossible rather than merely detectable.

R2 is what makes R1 hold at the wire boundary, where types do not reach and a hand-authored file can
say anything.

### R3 — wire-form class: the backstop

A verify class fires when a deserialised acceptance:

- **R3a** names more than one link; or
- **R3b** names a link the chain does not declare; or
- **R3c** names a link whose hash it does not sign.

R3 exists because R1 and R2 are properties of the in-memory type and the digest, and a store is a
directory of files that were not necessarily written by this code. Same reasoning as `ledger-core`'s
`L007`: the type cannot police bytes it did not produce.

### R4 — permission, stated so the refusal is not over-applied

The same identity signing **two different links, in two records**, is **permitted**.

Prohibiting it would be a competence judgement, and competence is routed, not checked (C-O-3). A core
that refused it would be pre-empting the very question the PRD says it must not pre-empt. If a corpus
supplies a predicate saying one actor may not ratify two named links, the core records the routing
and reports the violation; absent that predicate it reports nothing.

**R4 is the half most likely to be lost in implementation**, because R1–R3 all point one way and R4
points the other. It is written here as a rule rather than left as an absence.

## 3. The chain-length cases, worked

| Chain | Behaviour |
|---|---|
| **Length 0** | Refused at declaration. Not a claim. |
| **Length 1** | R1–R2 hold trivially. R3a can never fire. **R3b is not vacuous** — it still catches an acceptance naming a link that does not exist. The refusal is degenerate, not absent. |
| **Length 3 (the default)** | The A-6 case. R1–R4 as stated. |
| **Length 4+** | Unchanged. Nothing in R1–R4 counts links. |
| **Corpus B (length 2, no third far side)** | Signing "link 3" is refused by **R3b, as undeclared** — not as a failed discharge, and not as an escape. |

The corpus B row is the consistency check between this restatement and the ratified `inapplicable`
state: **the refusal side and the coverage side must give the same answer about a link that does not
exist.** They do. An undeclared link is unsignable (R3b) and uncountable (`inapplicable`), and
neither path can report it as undelivered or as escaped.

Had the two disagreed — had the refusal treated an undeclared link as a discharge failure while
coverage treated it as inapplicable — the same absence would have been a violation on one query and a
non-event on another. That is the audit-separation failure of PRD §4.6 arriving through the back door,
and it is the reason this table is here rather than assumed.

## 4. What the restatement costs

Honest accounting, since the variable chain is not free:

- **Per-link hashing** is more canonical-form surface than per-claim hashing: each link needs its own
  domain-separation prefix and its own field set, and each is a `CANONICAL_FORM`-class commitment
  that cannot be quietly changed later.
- **R3b requires the chain declaration to be resolvable at verify time.** A claim whose corpus
  declaration is missing cannot be checked for undeclared links — so a missing chain declaration must
  itself be a refusal, not a skip. Filed as **C-O-14**.
- **The default is now a default.** Three boundaries stop being structural, so anything that assumed
  three — a fixed-arity report, a three-column view — becomes wrong. Nothing built yet assumes it;
  this is a note for whatever is built next.

## 5. What this unblocks

C-O-7 and C-O-8 were held on the ground that a variable chain makes this refusal loose. R1–R4 hold
for any chain length including the degenerate cases, and do so without counting links: R1 and R2 are
per-link by construction, R3 is per-record, R4 is a permission. **The refusal does not become looser
as the chain becomes variable, because it was never a rule about the chain — it is a rule about a
single acceptance record.**

That is offered as the ground for lifting the hold. It remains the principal's to lift.
