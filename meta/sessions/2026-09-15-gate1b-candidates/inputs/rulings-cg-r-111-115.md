# Rulings CG-R-111 … CG-R-115 — Gate 1b, Gate A

**Issued by Emil, 2026-09-15.**

---

## CG-R-111 — Three arrived-input defects, all mine

**The hash.** The bootstrap cited `6e0992b1…`; the file delivered is `565cc7cc…`. I amended the rulings file to add CG-R-109 and CG-R-110 **after** citing its hash and did not re-cite. The session recording the received hash rather than the cited one is correct conduct and is the only reason the discrepancy is visible.

**The title.** The file is titled 105–108 and carries 105–110. Same cause: renamed on disk, not in the document.

**CG-R-99 to CG-R-104 never reached the session**, while the prompt cites CG-R-101 and CG-R-103. It complied from the prompt's restatement, which is the best available conduct and is not a substitute for the inputs.

**Ruled:** the received hash is authoritative and is what the arrival record carries. The title is corrected by appended note, not by amendment. CG-R-99 to CG-R-104 are supplied before Gate B.

**And a rule for me, since this is the third citation error in the programme** — the 78% hand estimate, the MediatR premise, now a stale hash: **a hash is cited from the artefact as it is delivered, not from a build step earlier.**

---

## CG-R-112 — A health probe is a candidate, and it is rejected at ratification

It is an external integration point under CG-R-105 — something outside the system initiates it. That it settles nothing is a judgement about the act, not about whether the point exists.

**Ruled: candidates are derived mechanically and judged at ratification.** A health probe appears in the set and is rejected with the reason *supplies no determination; liveness signal only*.

**Excluding it by definition would create an "infrastructure endpoints" category**, and that category grows. Every exclusion-by-kind is a place where something real gets filed under a label nobody revisits. The derivation stays mechanical; the judgement stays at acceptance, where it has a principal.

---

## CG-R-113 — BlazorAdmin is an actor, not an entry-point set

Its pages run in the browser and call A's API. They are entry points **of the Blazor application**, not of the solution under measurement.

And by CG-R-107 the Blazor client is itself an actor: it validates, retries, decides what to send. It is a resolver, so it is an actor, and it belongs as an **observed actor kind on the endpoints it calls** — not as a row in A's candidate set.

**Ruled: out of A's set.** Where the reader can see which endpoints it calls, that is observed actor evidence on those candidates.

---

## CG-R-114 — A candidate without a route has incomplete identity, and is labelled

Twenty-three of sixty-eight — a third of the set — are FastEndpoints whose route is a call argument the reader does not emit (L-EP-2). The verb is read; the path is not.

**Ratifying a candidate you cannot point at is not ratification.** But the integration point exists and dropping it would understate the surface by a third.

**Ruled:**

- They remain candidates, with **identity marked incomplete** and the missing field named.
- The route is **supplied by hand at ratification**, from the source. It is visible to a reader even where it is invisible to the reader, and supplying it is part of accepting the candidate — the same shape as the actor and scale slots.
- A candidate accepted without its route is a defect, not a shortcut.

---

## CG-R-115 — Ratification for A is a stand-in, graded, and covers a subset

**Nobody holds intent for eShopOnWeb.** Acceptance names the act, what it settles, and the principal — and for a reference sample, all three would be my guess at what someone else meant. R-A's ratification step exists precisely to restore the warrant that reading-from-code drops, and a stand-in ratifier restores less of it.

**Ruled, in two parts.**

**The resulting vocabulary is graded `stand-in`**, and every figure computed against it carries that grade. It is transport-informed guesswork by someone without intent, which is weaker than a measurement vocabulary and much weaker than an accrual one.

**And the ratified subset is the signal-bearing one, not all sixty-eight.** The thirty cross-type overlaps, plus whatever else I name, and the rest rejected with the reason *not ratified; no intent-holder available*.

**This is not a shortcut around §14.3's adoption cost.** It is the honest form for this codebase: a few acts specified, most entry points unmapped, coverage terrible — which §14.5 says is the correct early state and exactly what the flow should produce. Ratifying sixty-eight acts I would be inventing would give a full-looking vocabulary and a worse result.

**The cross-host overlap is the first one I want named.** The Web cookie logout and current-user actions sharing a path with the PublicApi JWT authenticate endpoint, across two hosts, is two auth schemes over what may be one act — which is the `channel` axis from CG-R-107 arriving on real code at the first opportunity.

---

## Also

**Recall 68/71 and precision 68/68**, with the three misses being endpoint-routing builder calls the reader records nothing for. Precision at 100% again, for the fourth measured component. Reported, not repaired: the instrument is closed under CG-R-103.

**The type-granular path limit** — the overlap measure can only discriminate across types, so a codebase whose acts live on shared types reads as fully merged — is a stated limit and needs its incidence per CG-R-77. On A, 220 of 250 pairs are within-type and structural; whether that ratio holds elsewhere is unmeasured.

**The first derivation running on a binary one edit behind the commit, discarded and re-run from the committed tree**, is CG-R-102's rule catching exactly what it was written for, three runs after it was written.

---

Register debt: CG-R-17 … CG-R-115. Mine.
