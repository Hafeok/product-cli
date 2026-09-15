# CG-R-104 (filed late) and rulings CG-R-116 … CG-R-118

**Issued by Emil.** CG-R-104 is reproduced as issued, with its supersession recorded. It is not rewritten.

---

## CG-R-104 — Gate 1b is deferred; Gate 3 runs next, greenfield

**Issued 2026-09-14. Superseded by CG-R-105, 2026-09-14. Filed 2026-09-15.**

> The PRD put Gate 1 first because it was written as a brownfield assessment tool. For the thing being tested, Gate 1 was the detour.
>
> The act vocabulary blocking §12.1 is eShopOnWeb's — a brownfield vocabulary for a codebase nobody here holds intent for. The greenfield vocabulary already exists: `ordering.eventmodel.yaml` has the acts, the facts and the positions, and `place-order.determinations.yaml` has determinations at an address. Gate 3 needs none of the delta, none of the reachability, none of eShopOnWeb.
>
> What it tests is the claim the whole thing rests on: whether act, fact and position are sufficient for an actor to build a conforming slice without asking what was meant.

**Supersession.** CG-R-105 unblocked Gate 1b by deriving a measurement vocabulary from entry points, which removed the reason for the deferral. Gate 3 remains bundled and unrun; that part of CG-R-104 stands as a statement of what is still outstanding.

**The filing defect is mine.** It was issued in conversation and never written down, which is the CI-1 defect — output that is not a repository object cannot be retrieved later — arriving on my side of the exchange rather than a session's. Third instance of the class in this programme.

---

## CG-R-116 — Overlap is evidence of shared realisation, not of a shared act

**This corrects what I told you.** I called the thirty cross-type pairs *the signal*. The session's own measurement does not support that reading, and L-EP-4 says why: the path is type-granular beyond its first hop, so two entry points calling the same service read as overlapping whether or not they settle anything in common.

**Ruled: overlap raises a question, it does not answer one.**

| | |
|---|---|
| **what overlap shows** | two entry points reach the same types — shared *realisation* |
| **what a merge requires** | two entry points settle the **same determinations** for the **same actor kind** — shared *act* |

Shared realisation is ordinary and mostly uninteresting: an identity subsystem serves many acts by design. **A merge is warranted only where the determinations coincide**, and that is a judgement at ratification, not a property the instrument can compute.

**Consequence for the worksheet.** *Signal-bearing* is the right label for the twenty-five — they are the rows that need a decision — but it must not read as *these are merges*. They are the rows where the instrument found something worth a human look, and most of them will resolve to distinct acts sharing a subsystem.

---

## CG-R-117 — The cross-host trio is not a merge

Two `UserController` actions and `AuthenticateEndpoint` share a path because they reach the same identity types. Under CG-R-116 that is shared realisation.

**They are distinct acts.** Establishing a session, ending one, and reading who the current actor is settle different things and have different consequences. No amount of shared `SignInManager` makes them one act.

**What is interesting about the trio is the channel, and it is the first real instance of CG-R-107.** The same act — establishing a session — is reached through a cookie scheme on one host and a JWT scheme on another. Two resolvers, two actors, one act. That is exactly what the `channel` extent axis was ruled in for, and it has arrived on real code at the first opportunity rather than as an argument.

**Tentative, pending the worksheet.** I have the trio's identity and not its determinations. If the two hosts settle different things about session establishment — different lifetimes, different claims, different revocation — then the channel axis carries real determinations and this is worth specifying first. If they settle the same things, it is one act travelling to both channels and the axis records that.

---

## CG-R-118 — The Blazor actor evidence is unavailable, and that is a stated gap

CG-R-113 said the Blazor client belongs as observed actor evidence *where the reader can see which endpoints it calls*. It cannot: 0 of 23, because the client's request calls never reach the reference graph, and the five `ReadAsStringAsync` calls that do carry no routes.

**Ruled: recorded as a gap with its incidence, not worked around.** The two `[Endpoint(Name=…)]` attributes naming endpoints by string are correctly reported and correctly not used — a string match would be inference from naming, which the prohibitions forbid.

The consequence is that A's actor evidence is thinner than the ruling anticipated, and any figure resting on observed actor kinds carries that.

---

## What I owe before Gate B

**`rulings-cg-r-99-103.md`** — attached. With CG-R-104 above, the citation gap closes.

**The worksheet, filled — and I need it delivered.** I cannot name acts for rows I have not seen: the paths, the types reached, and the read/write positions are what the naming rests on, and inventing them would be the fitting CG-R-115 was written to avoid.

On receipt I supply, for the twenty-five signal-bearing rows: `act`, `settles`, `principal`, actor kinds, population and rate as orders of magnitude, `channel` where it applies, and a merge decision per cross-type pair under CG-R-116's test. The forty-three keep the CG-R-115 default unless a row obviously warrants otherwise. The two probes stay rejected as pre-filled. **The fallback-file endpoint gets a decision from me rather than a rule** — no ruling names it because none should; it is one row and it wants a judgement, not a category.

Routes read from source are confirmed row by row against the file and line given, not accepted in bulk.

---

Register debt: CG-R-17 … CG-R-118. Mine.
