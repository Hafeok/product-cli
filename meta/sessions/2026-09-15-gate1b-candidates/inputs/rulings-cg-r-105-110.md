# Rulings CG-R-105 … CG-R-108 — the entry-point candidate vocabulary

**Issued by Emil, 2026-09-14.** These unblock Gate 1b without either party authoring a domain vocabulary.

---

## CG-R-105 — Entry points yield a measurement vocabulary, not an accrual vocabulary

An external integration point is where an actor outside the system initiates: a REST endpoint, a Razor page render, a queue subscription, a scheduled trigger. Each is evidence for an act, and the path from it is the slice's realisation.

**A library is not this case.** A library has no entry points of its own — it has a surface, and which parts are entry points is a property of whoever consumes it. Library work is boundary declaration over the *used* surface, per the earlier ruling, not act derivation.

**Ruled: two vocabularies, graded differently.**

| | Derived from | Good for | Not good for |
|---|---|---|---|
| **measurement vocabulary** | entry points | computing the delta, sizing the work | accruing determinations |
| **accrual vocabulary** | domain modelling, ratified by intent-holders | addressing determinations | — |

The measurement vocabulary is **transport-shaped**, and that is a stated defect rather than a detail: an address that is `POST /orders` does not survive a versioning or a transport change, because the address moves while the act does not. That is R-B arriving from the implementation side.

**Every figure computed against it is labelled transport-derived**, and the limit travels with it.

**CG-R-51 is partly superseded.** The day-one qualification claim returns in a bounded form: on day one you get **candidates and a structural delta**, not a domain assessment. That is a smaller claim than the original and it is defensible, which the original was not.

---

## CG-R-106 — A candidate is a prompt, not a draft

This is what keeps the exercise on the right side of R-D.

**Ruled:**

- A candidate carries the transport name, the path, the observed positions, and nothing else. **It does not carry the act's name, what it settles, or who answers.**
- **Accepting a candidate requires supplying those.** A tick that promotes `POST /orders` to an act creates a transport-shaped address that drifts on first contact. Per-candidate acceptance with naming is ratification; ticking a list is not, however many boxes it has.
- Rejection is recorded with a reason, so the pruning is checkable rather than trusted.

**Merges and splits are the signal, not the acceptance rate.** Several endpoints collapsing to one act, or one opening into several, are where transport shape and act shape disagree — and those are the seams. A near one-to-one acceptance means a codebase whose transport boundaries already match its acts, which is worth knowing and unusual. A high merge rate says the opposite, and it ranks where the specification work should start.

---

## CG-R-107 — The channel is arrangement, not identity

An Android client and an iOS client backed by the same user are two actors — **but not because identity is composite.**

Making the channel part of identity makes the actor set unbounded: user × platform × app version × locale, all "different actors" by the same argument, with nothing to stop it. Identity stays primitive.

**They are two actors because each is itself a resolver.** The Android app validates before sending, retries, decides what to include; so does the iOS app. Two distinct things that resolve choices. The human is a third actor and the accountable principal for both — which the allocation schema already carries, `carried_by` naming the executing actor kind and `principal` naming who answers.

**Determination variation by channel is extent, not actor proliferation.** `channel` joins the named extent axes: a determination binds at the act and travels to all channels, binds to one, or is silent on the axis. Four states, already built, and it handles app versions without inventing an actor per release.

**Ruled: `channel` is a named extent axis from the start**, not deferred to the first engagement that needs it. Adding an axis later leaves every prior determination *silent* on it and unrecoverable as to whether that silence was deliberate — the one loss the extent design exists to prevent.

---

## CG-R-108 — Candidates carry observed actor evidence and unfilled slots for the rest

Actors are where the required complexity lives, and none of it is currently on the candidate.

**Observed, from code — evidence, labelled as such:** authorisation attributes and roles, authentication schemes, anonymous access, any client-identity check on the path.

**Supplied at acceptance, and required for it:**

- **expected actor kinds**, referencing Layer 1 kinds rather than free text, or the vocabulary goes uncontrolled within a week
- **population** — how many distinct actors of this kind exist
- **rate** — calls per actor per unit time

**Population and rate are separate numbers driving different determinations.** Population drives tenancy, isolation and authorisation granularity. Rate drives concurrency, duplicate delivery, idempotency and staleness tolerance. Collapsing them into "volume" loses which determinations the act needs.

**Orders of magnitude, not figures.** 1, 10, 10³, 10⁶. A precise number is false precision, is stale within a quarter, and changes no determination the magnitude did not already change.

**Scale is a determination input, not a non-functional footnote.** The same act at one actor and at ten thousand is a different specification — BC-3 duplicate delivery, BC-4 concurrency, PC-3 staleness. Putting scale on the candidate puts it in front of whoever writes the determinations, before they write them.

**Unfilled is not empty.** An absent actor list reads as *not stated*, never as *no actors*. Same distinction as silent versus does-not-travel, and the same reason: absence and decision must not look alike.

---

## CG-R-109 — Supported throughput is a determination, not a ground slot

Rate and supported throughput are different objects and must not share a field.

| | Kind | Wrong how |
|---|---|---|
| **rate** | ground — what actors will do | an estimate can be inaccurate |
| **supported throughput** | **determination** — what the system commits to serving | a commitment can be unmet, and someone answers |

**Ruled: the candidate carries three kinds of field**, marked as such:

1. **observed** — read from code, labelled
2. **unfilled ground slots** — expected actor kinds, population, rate
3. **invited determinations** — supported throughput, and anything else the candidate prompts but does not supply

The third kind is new, and it is the first place a candidate points at a determination rather than at evidence. Keeping it distinct is what stops acceptance quietly authoring determinations, which R-D and CG-R-106 both forbid.

**Throughput's form.** A bare number is not checkable. It carries:

- **sustained** and **peak**, separately
- a **window** — "1,000/s" is meaningless without a duration
- **behaviour above the limit**: reject, queue, shed, degrade

**The behaviour above the limit is the more consequential determination.** It exists whether or not anyone states it, it has real consequence, and it is exactly the class that gets resolved silently by whoever is writing the code at the time. **A throughput figure with no stated behaviour above it is a coverage claim with no uncovered set**, and PR-3 forbids that shape everywhere else.

Throughput is **operationally closed** once it carries these fields — it can be tested against, unlike a population estimate, which is why it is worth having as a determination rather than as a fourth estimate.

---

## CG-R-110 — Scale sets the consequence of escape, not the demand

`[PROPOSED]`, unfalsified, filed so it is on the record rather than in conversation.

**Scale does not create determination demand.** At population 1 the tenancy decision still exists and has an answer; nothing forces anyone to make it. At 10⁶ the same decision is forced, because escaping it now has consequence. The demand was always there and latent, which keeps conservation intact: demand is allocated, not generated.

That is also why scale belongs on the candidate. It does not say which decisions exist. It says **which uncovered categories matter here**.

**Two refinements on the driver.**

**Kinds, not count.** A million identical actors demand less than three actor kinds with different capabilities and different carried ground. An Android app, an iOS app and a partner integration against one endpoint generate more determinations between them than a million users of any one. Heterogeneity is the driver; count is a proxy for it and a poor one.

**Step function, not proportion.** 1 → 2 adds tenancy. 10³ → 10⁶ may add nothing. The thresholds are what matter, which is why orders of magnitude were the right form.

**One consequence for the delta's reporting.** If scale sets the consequence of escape, **the uncovered list is not flat**. The same uncovered category at population 1 and at 10⁶ are different findings, and the report weights by the candidate's declared scale rather than counting gaps equally. That is a change to how the delta reports, not to what it measures.

**No falsifier.** It is a claim about where demand becomes forced and nothing in the programme tests it. Filed as a candidate hypothesis beside actor-proximate accrual.

---

Register debt: CG-R-17 … CG-R-108. Mine.
