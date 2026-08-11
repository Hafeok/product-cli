# Basis-quality audit — pre-format-5 decisions (2026-08)

**What is under test.** `DDD-method-06` — *mandatory claim-basis manufactures
weak claims* — filed `projected` when typed basis (`dec/ddd/typed-basis`,
decision format 5) was ruled, with this falsifier:

> An audit of basis quality after typed basis (format 5) ships: if the
> pre-format-5 decisions' claim bases audit as genuine — each basedOn claim
> would have been filed on its own merits, none exists only to satisfy rule 3
> — the manufactured-claims mechanism did not operate here and the claim dies.

This document is that audit. **Section 1 was written and committed before any
rationale text was read or any classification made.** Nothing in it was
adjusted after results were seen; if the criteria look badly chosen in
hindsight, that is a finding about the pre-registration, not a licence to
re-cut it.

---

## 1. Method, as pre-registered

*(Committed before the data was gathered. Fixed.)*

### 1.1 Population

Every decision file under `.ddd/decisions/` whose `format:` field is **less
than 5** — i.e. filed before decision format 5, the format that made basis
typed. In such a file every `based_on` entry is an untyped `claim: <id>`
reference, because rule 3 admitted nothing else.

Format-5 decisions are **excluded**: they were filed after the affordance
existed, so they cannot exhibit the mechanism under test.

The unit of the headline reading is the **decision**. The unit of the table is
the **citation edge** (decision → claim).

### 1.2 Per cited claim, record — mechanically

| Signal | How it is computed |
|---|---|
| **Co-filing** | The commit that introduced the claim file (`git log --diff-filter=A -1`) compared with the commit that introduced the citing decision file. Same commit → `co-filed`. Different → `pre-existing`, with both hashes recorded. |
| **Citation count** | Number of **distinct decisions** (any format, whole store) whose `based_on` names the claim. `1` = single-purpose. |
| **Falsifier** | `present` / `absent` / `unfalsifiable`. See 1.3. |
| **Status** | The claim's `status:` field verbatim. |
| **Evidence** | Whether an `evidence:` field is present. |
| **Basis classification** | What the basis actually was, read from the **decision's own `rationale` / `title` text** — never from the claim, never from the `type` field. One quoted phrase per row, the phrase that decided it. See 1.4. |

### 1.3 Falsifier grading — criteria fixed in advance

A falsifier is **`present-but-unfalsifiable`** if no observation of this repo,
its toolchain, or its use could ever produce the stated condition. Three
disqualifiers, one word recorded per claim:

- **`restatement`** — it names no observable event, only the negation of the
  statement ("if X turns out not to hold").
- **`vapour`** — it is conditioned on an artifact or milestone that does not
  exist and has no scheduled existence, so the observation can never be made.
- **`taste`** — it requires a judgement of quality or intent with no stated
  threshold, so no observation settles it.

Everything else with a `falsifier:` field is **`present`**. A falsifier that is
merely *unlikely to fire*, or that names a future-but-scheduled milestone, is
`present` — hard to falsify is not unfalsifiable.

### 1.4 Basis vocabulary — definitions fixed in advance

Read from the decision's rationale text. The question is *what the rationale
actually argues from*, not what it cites.

- **claim** — the rationale argues from a **contestable proposition about the
  world** that could be shown false by observation, and the decision follows
  from it. Form: "X does/does not Y", "X is sufficient for Y".
- **constraint** — the rationale argues from something the project **cannot
  change**: a language, tool, or format property; a host-system fact. Form:
  "Rust has no X", "the format requires Y".
- **mandate** — the rationale argues from an **authority requiring it**: a
  principal's ruling, a spec, an upstream or house rule. Form: "ruled at
  review", "the PRD requires", "policy is".
- **preference** — the rationale argues from **taste, ergonomics, or style**,
  with no falsifiable proposition. Form: "cleaner", "reads better", "we prefer".
- **experiment** — the rationale is explicitly **provisional, adopted to find
  out**. Form: "try it and see", "for now".
- **risk-acceptance** — the rationale **acknowledges a known cost or gap and
  prices it**. Form: "priced, not paid", "accepted cost", "we take the hit".
- **indeterminate** — the text supports two or more of the above and **no
  phrase discriminates them**. Recorded as a result, not resolved by guessing.

A row where the classification was genuinely close is additionally marked
**borderline** and listed in §5 with its quote, for the principal's ruling. It
still receives its best-reading classification in the table; borderline is an
annotation, not a third value.

### 1.5 Weak-claim composite — fixed in advance

A cited claim is **weak** iff it meets **two or more** of:

- **W1** — co-filed with the citing decision in the same commit;
- **W2** — cited by exactly one decision;
- **W3** — falsifier `absent` or `unfalsifiable`;
- **W4** — status `projected` **and** no evidence attached.

Two-of-four, not one: any single signal has an innocent reading (a claim filed
alongside its first consumer is normal practice; a claim cited once may simply
be young). Two together is the pattern the claim predicts.

### 1.6 The reading

A citation edge is **manufactured** iff **both**:

- the citing decision's basis classification is **not `claim`** (it is mandate,
  preference, experiment, risk-acceptance, or constraint), **and**
- the cited claim is **weak** by §1.5.

A **decision** counts as a **manufactured-basis decision** iff **every** claim
it cites is manufactured. The `every` is deliberate and tight: if a decision
cites one genuine claim alongside one weak one, rule 3 was already satisfied by
the genuine one and nothing needed inventing. Only a decision with no genuine
claim to stand on was *forced* to manufacture.

**The audit's result is that count, over the population.**

### 1.7 Verdict bands — fixed in advance

Chosen before the count was known:

| Count of manufactured-basis decisions | Verdict | Proposed status |
|---|---|---|
| **0** | The falsifier fired exactly as written. | `retired` |
| **1–3** (≤ ~7%) | Mechanism operated, but marginally. Report it as *held weakly* and say plainly that the typed-basis ruling rests mostly on its ergonomic argument. | `reported`, weakly |
| **≥ 4** (≥ ~10%) | Prediction held. | `reported` |

**Underpowered** is claimed only if **both**: fewer than 10 decisions in the
population classify as non-`claim` basis, **and** more than 30% of
classifications come out `indeterminate` — i.e. the rationale texts cannot
discriminate the types at all. Population size alone is not grounds; 41 is not
a small N for this question.

### 1.8 Constraints on this audit

Nothing in the store is changed. No re-typing of bases, no claim edits, no
retirements, no status moves, no cleanup of anything noticed. Every finding in
§6 is a **proposal for the principal's ruling**, filed unactioned.

---
