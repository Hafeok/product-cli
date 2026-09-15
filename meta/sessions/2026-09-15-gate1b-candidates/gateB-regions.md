# Gate B — the delta over the ratified set, held for ratification

**Status: `[PROPOSED]`, 2026-09-15.** Every figure is **transport-derived** (CG-R-105) and
**stand-in** (CG-R-115). Full print: `measurement/gateB-A-regions.txt`; every entry point,
act and figure in `measurement/gateB-A-regions.json`.

## 1. Instrument record

`measurement/gateB-instrument.txt`: gates run **before** measurement (CG-R-102) — `cargo t`
52 binaries ok, 0 failed; clippy ok; xtask ok. Commit `f700915`, working tree 0 uncommitted
paths, binary sha256 `8474aa69b361f549`, inventory A v6 `3197d7e07c5a0b46` (the run-8 artefact;
reader not re-run), ratification `dff974baac758904` (Emil's, stand-in graded). The run-8 A reach
output regenerated **byte-identical**. One run; nothing discarded.

## 2. The vocabulary — 20 acts over 22 entry points

From the worksheet's accepted rows. Positions by proxy P-EP-4 (`gateB-criterion.md` §1):

| Act | Entry points | Channel | reads | writes |
|---|---|---|---|---|
| AddItemToBasket, ReadBasket, ReadBasketSummary, SetBasketItemQuantities | 1 each | web-cookie | Basket, CatalogItem | Basket, CatalogItem |
| CreateCatalogItem, UpdateCatalogItem | 1 each | api-jwt | CatalogItem | CatalogItem |
| ReadCatalogItem, ReadCatalogItemList | 1 each | api-jwt | CatalogItem | — |
| EndSession | **2** (cookie action + Logout page) | web-cookie | — | — |
| ReadUserDetails | **2** (by id, by name) | api-jwt | — | — |
| ConfirmEmailAddress, EstablishSession, ReadCurrentActorIdentity, RegisterAccount, DeleteRole, DeleteUser, ReadUserList, ReadUserRoles, RemoveUserFromRole, SetUserRoles | 1 each | as ratified | — | — |

**Twelve of twenty acts have no position under the proxy.** Their state lives behind
`UserManager<T>`, `RoleManager<T>` and `SignInManager<T>`, which P-EP-4 does not read. That is
the proxy's blindness (Gate A §4) arriving in the vocabulary: the identity half of A's acts
carries no facts, so nothing on their paths can be covered or spanned by them.

## 3. The regions — 68 entry points

| Region | Count | by kind |
|---|---|---|
| declared | 22 | controller 2 · page 6 · view-component 1 · fastendpoints 13 |
| declarable | 5 | page 4 · fastendpoints 1 |
| unstructured | 7 | controller 2 · page 3 · fastendpoints 2 |
| **no facts under the proxy** | **34** | controller 21 · page 6 · fastendpoints 7 |

By host: Web declared 9 · declarable 4 · unstructured 5 · no-facts 27; PublicApi declared 13 ·
declarable 1 · unstructured 2 · no-facts 7. The three hand-supplied rows have no path and sit
outside the regions.

**Declarable (5)** — `Admin/EditCatalogItem` GET and POST, `Identity/Account/Login` GET and
POST, `DeleteCatalogItem`. Each has a clean path whose only fact under the proxy is
`CatalogItem` or `Basket`, so *every* accepted act that touches that entity "covers" it: the
Login page reads as declarable for the four basket acts because its path reaches
`BasketViewModelService` and the basket acts read `Basket`. That is CG-R-52's known divergence
verbatim — a shared entity type read as a shared act — and it says the separator, on P-EP-4
facts, cannot tell *which* act an entry point is declarable for, only that one exists. The
covering-act lists are printed and should be read as "some act touches the same entity".

**Unstructured (7)** — the ordering side and the catalog lists: `Order/Detail`,
`Order/MyOrders`, `Basket/Checkout` GET and POST, the home page `Index`, `CatalogBrandList`,
`CatalogTypeList`. Six spanning types sit on their paths:

| Spanning type | Facts under P-EP-4 | Acts touching |
|---|---|---|
| `OrderService` | Basket, CatalogItem, **Order** | the eight basket/catalog acts — none covers `Order` |
| `GetMyOrdersHandler`, `GetOrderDetailsHandler` | Order | none |
| `CatalogViewModelService` | CatalogBrand, CatalogItem, CatalogType | the eight — none covers brand or type |
| `CatalogBrandListEndpoint`, `CatalogTypeListEndpoint` | CatalogBrand / CatalogType | none |

Separator incidence: **6 spanning of 16 path types with facts** (171 path types in all).
`OrderService` was the predicted spanning type; the reason is exact — `Order` is written
nowhere in the accepted vocabulary, because no ordering entry point was in the signal-bearing
subset, so the type that places an order spans the basket acts and an act nobody named.

**No facts under the proxy (34)** — every `ManageController` action (21), the Identity pages
not accepted, the role and user endpoints not accepted. The largest region by far, as both
predictions said. **This region is not a finding about A; it is the proxy's field of view.**

## 4. The split, with the bound in CG-R-120's form

Walk from 28 roots of the 22 accepted entry points through the hosts' sites, O-17 off, over
the 301 production types that are not an accepted entry point's own type:

| | Count | Share |
|---|---|---|
| reachable-undeclared | 66 | **21.9%** |
| unresolved-undeclared | 0 | 0 |
| isolated-undeclared | 235 | 78.1% |

By project: ApplicationCore 22 reachable / 22 isolated; Infrastructure 7 / 21; PublicApi
24 / 40; Web 9 / 83; BlazorShared 2 / 17; BlazorAdmin 2 / 46 (two client types reached through
registrations the Web host makes); AppHost 0 / 5.

**Error bound (CG-R-120, in force): 29 unfollowed of 48 composition edges = 60.4 points** —
composition: unresolved 3, registration-not-read 26. The table deciding the 26 is unvalidated
(CG-R-101): two unvalidated things in series, both labelled.
**Prior form (CG-R-89), beside it: 0 unscored of 48 = 0.0 points.**

**§12.1**, read against the split it asks about: 21.9% reachable, bound 60.4 points, distance
to ~90 is 68.1 points — the split discriminates *by the letter of the test*, by 7.7 points.
Read plainly: the accepted roots could reach anything from 21.9% to 82% of the undeclared
types depending on 26 registrations the reader cannot read, most of them the identity stack
behind the account and user acts. The isolated region (235 types) is real for the ordering
side, the manage pages and the client, which no unfollowed edge leads toward; it is **not**
established for the identity-adjacent types the 26 edges could open. §12.1 does not fire, and
the bound says how little that is worth here.

## 5. The two predictions, scored — each against its own form

| | Emil (CG-R-119) | Actual | Session | Actual (its form) |
|---|---|---|---|---|
| reachable-undeclared | 30–50, centre 38 | **21.9 — outside; the stated falsifier fired** (under 25: the accepted roots do not reach the shared core as expected) | 18–32, centre 24 | **21.9 — inside** |
| error bound | 8–20 points (CG-R-120 form) | **60.4 — outside, three times the top of the band** | 2–6 points (CG-R-89 form) | **0.0 — outside, below the band** |
| largest region | no facts under P-EP-4 | **yes**, 34 of 46 | no facts, ~32 of 46 | **34 — inside** |
| declarable / unstructured | — | — | 5–12 / 2–8 | **5 / 7 — inside** |
| spanning types | — | — | 2–6, `OrderService` among them | **6, `OrderService` among them — inside** |

**On the reachable share.** Emil's reasoning was that the accepted subset reaches the shared
core — entities, repositories, specifications, the context — and it does: ApplicationCore is
half reachable and `CatalogContext`, `EfRepository`, the specifications are in the 66. What the
reasoning over-counted is how small the shared core is relative to the 301: Web alone holds 92
undeclared types, 83 of them isolated (view models, the manage pages, extensions,
configuration), and the client holds 48. The concentration bought exactly the core and nothing
of the periphery, which is what the session's paths-summed reading expected.

**On the bound.** Both were wrong, in opposite directions, for the same reason: under CG-R-89
nothing on the accepted paths is partial, boundary or excluded — every edge either resolves or
is unfollowed — so the prior form is 0 and the CG-R-120 form is everything the walk could not
follow. Emil's falsifier ("2–6 with not-read included means the not-read edges sit on paths
reachable by another route") did not fire; the opposite holds — 26 not-read edges of 48 is the
identity stack, and nothing else reaches what lies behind it. **The bound is the finding**: on a
vocabulary whose acts are half identity, the reachable/isolated split is decided by a table
nobody has validated, and CG-R-120's rule is what makes that visible rather than a 0.0.

## 6. §12.2 — do the regions separate?

They separate **mechanically** — no per-symbol judgement was made; every row's region follows
from the rule and prints its reason. But the separation applies to 12 of 46 undeclared entry
points; 34 are outside the proxy's field of view, and among the 12, *declarable* cannot say for
which act (§3). The honest reading is the one the session predicted: the three-region report is
a two-region report — *has facts under the proxy* / *has none* — with a mechanical but weakly
discriminating split inside the first. That is a statement about P-EP-4, not about A.

## 7. Findings

- **F-EP-3** stands as established (`gateB-criterion.md` §0): the first codebase finding.
- **F-EP-4 — the identity half of the vocabulary carries no facts.** 12 of 20 acts, 34 of 46
  undeclared entry points, and 26 of 29 unfollowed edges are the identity stack. Every figure
  above is dominated by what P-EP-4 does not read. A fact proxy that reads `UserManager<T>`'s
  members by symbol id would change all three; that is instrument work and is not undertaken
  (CG-R-103).
- **F-EP-5 — `Order` is written by no accepted act**, so `OrderService` spans, and Checkout,
  Order/Detail and Order/MyOrders are unstructured. The ordering side was outside the
  signal-bearing subset because the overlap measure never flagged it (its types are shared with
  nothing accepted). A consequence of CG-R-115's selection rule, recorded.
- **The declarable lists over-cover** (§3): a shared entity reads as a shared act. CG-R-52's
  divergence, now with an incidence: 5 of 5 declarable rows list four or more covering acts.

## 8. What this does not establish

The vocabulary is transport-derived and stand-in: no domain modeller has seen it. Actor and
scale slots are unfilled on every accepted row (stated reasons). The table behind 26 of 29
unfollowed edges is unvalidated. A is a maintained reference sample, not the brownfield codebase
the delta was designed for. Nothing here is a determination.

## 9. Weakest point

**The bound.** At 60 points on a 22-point figure, the split's error bar is nearly three times
its value. §12.1 does not fire only because 90 is far away. Any reading of "78% isolated" as a
property of A is unsupported for the identity-adjacent types; it is supported for the ordering
side, the manage pages and the client only because nothing unfollowed points at them, and that
argument is the session's, not the instrument's.

## 10. Held

Gate C — the report — waits on Emil's reading of this gate: whether the regions as computed are
the split §12.1 asked about, and whether the bound's composition is to be reported as it is or
split further (not-read edges whose targets are external framework types cannot reclassify an
in-solution type directly, but the session has not established that and does not claim it).

---

## Appended — 2026-09-15, rulings CG-R-121 … CG-R-124 applied; run 2

**Nothing above is amended.** §4 above reads §12.1 as "does not fire, by 7.7 points"; **CG-R-122
rules that it fires** — the bound (60.4) is read against the effect it qualifies (21.9), never
against a constant, and the residual numeric form is struck. The instrument now prints it so.
CG-R-123: the bound's composition carries a third line — unfollowed edges with external
targets: **26 of 29**, all of the registration-not-read edges; reported, never subtracted.
CG-R-124: every reachable/isolated figure now carries the subset it was computed from,
`stand-in (CG-R-115), 22 of 68 entry points`.

**Run 2** (`measurement/gateB-r2-*`, commit `8246e6a`, gates before measurement green, tree
clean, regression byte-identical) differs from run 1 only in those lines; no figure moved. The
Gate C report (`gateC-report.md`) is written from run 2 and leads with CG-R-124's reading.

CG-R-121 scores the predictions as §5 above has them, and names the cause both bound
predictions share: a quantity whose composition neither party had established.
