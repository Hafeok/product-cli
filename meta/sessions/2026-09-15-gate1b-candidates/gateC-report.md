# Gate C — the report: Gate 1b on eShopOnWeb

**Status: `[PROPOSED]`, 2026-09-15.** The session proposes; nothing here is ratified by it.
Every figure is **transport-derived** (CG-R-105) and **stand-in** (CG-R-115), and every
reachable/isolated figure states the subset it was computed from (CG-R-124). The instrument
record for the figures below is `measurement/gateB-r2-instrument.txt`: gates before
measurement green (CG-R-102), commit `8246e6a`, tree clean, the run-8 reach output
byte-identical, ratification `dff974ba…`.

## 0. The reading to lead with (CG-R-124)

**The split measures the ratified subset as much as it measures A.** 78% of A's undeclared
types are not *unspecified*; they are *not reached by the twenty acts Emil chose to ratify*,
and a different subset gives a different split. In substance, this gate measured **the
non-identity half of A, and did not measure the identity half at all**: twelve of the twenty
acts carry no position because their state lives behind `UserManager`, `RoleManager` and
`SignInManager`, which the fact proxy does not read, and 26 of the 29 unfollowed edges are that
same stack. The vocabulary is `stand-in, 22 of 68 entry points`, and that label travels with
every number that follows.

## 1. The candidate set, with the merges

68 candidates, derived mechanically from A's external integration points (`gateA-candidates.md`):
25 controller actions, 19 Razor page handlers, 1 ViewComponent, 23 FastEndpoints endpoints, 0
hosted services; plus 3 integration points the instrument cannot see (two health probes and the
fallback file, L-EP-1), hand-supplied. Recall 68 of 71 against the hand enumeration, precision
68 of 68. 23 candidates had incomplete identity — the route is a call argument the reader does
not emit — and were completed by hand at ratification, confirmed row by row (CG-R-114); one
failed confirmation and is deferred (F-EP-3 below).

**The merge signal and what it turned out to be.** The instrument found 30 cross-type path
overlaps (Jaccard ≥ 0.5) among 250 overlapping pairs; the other 220 are handlers on one type,
structural (L-EP-4). Under CG-R-116 an overlap is evidence of *shared realisation*, not of a
shared act, and the ratification bore that out:

| | Count | What it was |
|---|---|---|
| overlapping pairs across types | 30 | flagged for a decision |
| merges found at ratification | **2** | `EndSession` over the cookie logout action and the Logout page POST; `ReadUserDetails` over the user lookups by id and by name |
| overlaps that were not merges | the rest | the cross-host trio (three acts sharing identity types, CG-R-117); the basket page and the basket view component (two read models over the same facts); the catalog and user endpoint families |

The cross-host trio is the first real instance of CG-R-107's `channel` axis: the same act,
establishing a session, reached through a cookie scheme on one host and a JWT scheme on the
other. The cookie-channel login page was not in the signal-bearing subset — its types are
shared with nothing accepted — so the axis is half-populated in this ratification.

## 2. The three regions, by cluster

The ratified vocabulary: 20 acts over 22 entry points. Undeclared entry points: 46.

| Region | Web | PublicApi | Total | controller · page · view-component · fastendpoints |
|---|---|---|---|---|
| declared | 9 | 13 | **22** | 2 · 6 · 1 · 13 |
| declarable | 4 | 1 | **5** | 0 · 4 · 0 · 1 |
| unstructured | 5 | 2 | **7** | 2 · 3 · 0 · 2 |
| no facts under the proxy | 27 | 7 | **34** | 21 · 6 · 0 · 7 |

- **Declarable (5)**: the admin catalog-item page (GET, POST), the login page (GET, POST),
  the delete-catalog-item endpoint. Every one lists four or more "covering" acts, because its
  only fact under the proxy is `CatalogItem` or `Basket` and every basket and catalog act
  touches those entities. CG-R-52's known divergence — a shared entity read as a shared act —
  now with an incidence: 5 of 5. The region says *some* act touches the same entity; it does
  not say which act the entry point belongs to.
- **Unstructured (7)**: the ordering side (`Order/Detail`, `Order/MyOrders`, `Basket/Checkout`
  GET and POST), the home page, the catalog brand and type lists. Six spanning types sit on
  their paths, `OrderService` first: it holds `Basket`, `CatalogItem` and `Order`, and `Order`
  is written by no accepted act. Separator incidence: 6 spanning of 16 path types with facts.
- **No facts under the proxy (34)**: every `ManageController` action, the Identity pages not
  accepted, the role and user endpoints not accepted. **This region is the proxy's field of
  view, not a property of A.**

**§12.2 — do the regions separate?** Mechanically, yes: no per-symbol judgement was made, and
every row prints its rule and reason. In substance, the three-region report is a two-region
report — *has facts under P-EP-4* / *has none* — with a mechanical but weakly discriminating
split inside the first, because 34 of 46 undeclared entry points fall outside the proxy's field
of view and the declarable region cannot name its act. The prediction on the clause (both
parties: the largest region would be *no facts*) held.

## 3. Both ratios, with the error bound

Computed from the subset **[stand-in (CG-R-115), 22 of 68 entry points]**, over the 301
production types that are not an accepted entry point's own type, by one walk from the 28
roots of the accepted entry points through the hosts' registration sites, O-17 off:

| | Count | Share |
|---|---|---|
| reachable-undeclared | 66 | **21.9%** |
| unresolved-undeclared | 0 | 0 |
| isolated-undeclared | 235 | **78.1%** — not reached by the ratified subset, which is not the same as unspecified |

Reachability propagates through shared infrastructure and not across sibling features
(CG-R-121): the entities, specifications, interfaces and the data context are reachable
because everything uses them; Web's 83 isolated types are page models, view models, the manage
pages and extensions that hang off entry points nobody ratified; the client's 46 are reached
by nothing on the server side.

**Error bound (CG-R-120, in force): 29 unfollowed of 48 composition edges = 60.4 points.**
Composition, reported beneath and never subtracted (CG-R-123):

| | Edges |
|---|---|
| unresolved | 3 |
| registration-not-read | 26 — decided by the table that is **unvalidated** (CG-R-101) |
| of the 29, with external targets | 26 — `UserManager<T>`, `SignInManager<T>` and their kind; `ApplicationUser`, the identity context and any custom store sit behind them |

The prior form (CG-R-89, unscored fraction) is 0.0 beside it: nothing on the accepted paths is
partial, boundary or excluded, so every edge either resolves or is unfollowed. **Two unvalidated
things stand in series — the registration table and P-EP-4 — and the bound is built from a
category the table decides.**

## 4. §12.1's disposition — it fires (CG-R-122)

A 60.4-point bound on a 21.9-point figure: the bound is nearly three times the effect it
qualifies. The accepted roots could reach anything from 22% to 82% of the undeclared types
depending on 26 registrations the reader cannot read. **The reachable/isolated split on A
cannot be trusted as currently instrumented**; the named cause is the unvalidated table plus
P-EP-4's blindness, both known and labelled before the run. The run measured how much they
matter: 60.4 points. No numeric threshold was read (CG-R-89's fire/clear form is retired; the
bound is read against the effect, never against a constant).

Firing does not mean the work was wasted. The isolated region is real for the ordering side,
the manage pages and the client — nothing unfollowed points toward them — and that argument is
the session's, not the instrument's; it is not established for any identity-adjacent type.

## 5. Predictions, scored (CG-R-121)

| | Emil | Session |
|---|---|---|
| reachable-undeclared | 30–50, centre 38 → **21.9, outside; falsifier fired** | 18–32, centre 24 → **inside** |
| error bound | 8–20 (CG-R-120 form) → **60.4, outside** | 2–6 (CG-R-89 form) → **0.0, outside** |
| largest region | no facts → **held** | no facts, ~32 of 46 → **held** (34) |
| declarable / unstructured / spanning types | — | 5–12 / 2–8 / 2–6 → **5 / 7 / 6, all inside** |

Both bound predictions failed for one cause — a quantity whose composition neither party had
established, the same shape as CG-R-63.

## 6. What this does not establish

- **No domain modeller has seen the vocabulary.** It is transport-derived, and Emil ratified it
  as a stand-in with no intent for eShopOnWeb; the acts are his guess at what someone else
  meant. Every figure carries the grade.
- **The actor and scale slots are unfilled** on every accepted row, with the reason stated:
  A has no deployment, and a stand-in cannot invent population, rate or a throughput
  commitment. CG-R-110's scale-weighted uncovered list cannot be computed; any figure resting
  on scale is unavailable, not zero. The Blazor client as observed actor evidence: 0 of 23
  endpoints, a stated gap (CG-R-118).
- **A is a maintained reference implementation**, not the brownfield codebase the delta was
  designed for.
- **The identity half is unmeasured** (§0). Twelve of twenty acts carry no position; 34 of 46
  undeclared entry points show no fact; 26 of 29 unfollowed edges are the identity stack.
- **The registration table is unvalidated** (CG-R-101 outstanding) and decides 26 of the 29
  edges in the bound.
- Nothing here is a determination, a slice, or a specification of A.

## 7. Findings about the codebase, distinguished from findings about the instrument

**About A:**
- **F-EP-3** — `src/PublicApi/UserManagementEndpoints/UpdateUserEndpoint.cs` declares a class
  named `UpdateRoleEndpoint` (line 13) routing `PUT api/users` (line 17). The class name
  disagrees with its file name; a second, unrelated `UpdateRoleEndpoint` exists under
  `RoleManagementEndpoints`. Found by confirming a route row by row (CG-R-114). The
  programme's first codebase finding.
- **F-EP-5** — `Order` is written by no accepted act; `OrderService` spans the basket acts and
  an act nobody named, and the ordering side reads as unstructured **because of a ratification
  choice** (CG-R-124), not because of anything about the code.
- The ratifier's own findings on the worksheet: `ConfirmEmailAddress` is a GET that changes
  state (transport verb and act disagree; the act wins, R-B); three of four subset rejections
  are GET renders whose act is on the POST.

**About the instrument:**
- **F-EP-1** — primary constructors carry the type body's references (23 of 23 endpoint
  constructors); no effect on composition edges. Reported, not repaired (CG-R-103).
- **F-EP-2** — endpoint mapping is invisible to the reader (3 of 71 integration points; every
  routing and authorisation convention configured there).
- **F-EP-4** — P-EP-4 is blind to identity state; it dominates every Gate B figure.
- The declarable region over-covers (5 of 5); the path is type-granular beyond its first hop
  (220 of 250 overlapping pairs within one type, L-EP-4).

## 8. Weakest point

**The bound, and what stands behind it.** 60 points of uncertainty on a 22-point figure,
built from 26 edges whose classification a table nobody has validated decides, over a proxy
that reads none of the identity state those edges guard. The one statement about A that
survives is the isolated ordering side, the manage pages and the client — and that rests on the
absence of unfollowed edges toward them, an argument the session makes and the instrument does
not.

## 9. Outstanding, and stop

- **CG-R-101** — the table's ground truth, the one permitted piece of instrument work, with
  `AddHttpClient<T>` landing with it. Not done in this gate. Gate B's bound is its first
  consumer and would change with it.
- **Gate 3** (CG-R-104) — greenfield, bundled, unrun.
- **Solution B** — out of scope throughout.
- **CG-R-107** — the channel axis has its first real instance (the cross-host session acts)
  and no determination yet written against it.

Gate 1b is complete as specified: the candidate set (Gate A), the delta over the ratified set
(Gate B), and this report (Gate C). The session stops here.

---

## Appended — 2026-09-15, closed under CG-R-125

**Nothing above is amended.** Emil closed Gate 1b as specified. Two corrections to the reading
of §7 above, recorded here rather than by rewriting it:

- **F-EP-3 and F-EP-5 carry different weights and do not belong under one heading.** F-EP-3 is a
  finding about A — small, real, a class name disagreeing with its file name. F-EP-5 is **not a
  finding about A**: the ordering side reads as unstructured because no ordering entry point was
  ratified, a consequence of the subset (CG-R-124). §7 lists both under "About A"; read F-EP-5
  as a finding about the ratification.
- What Gate 1b established: the entry-point vocabulary works mechanically (68/71, 68/68);
  ratification produces what derivation cannot (two merges, one deferral that found a defect,
  three rejected renders); the split fires §12.1 with a bounded, named cause — the instrument
  is blind to exactly the subsystem carrying most of A's undeclared edges. What it did not:
  anything about A's specification coverage, anything about the identity half, anything about
  whether the flow works for anyone who did not design it.

Order of the outstanding work (CG-R-126): Gate 3 greenfield; CG-R-101's table ground truth;
the flow session; the notation falsifier. B out of scope. The session's records end here.
