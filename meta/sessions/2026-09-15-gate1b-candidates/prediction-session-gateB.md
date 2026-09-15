# Prediction — the session's, Gate B, committed before the run

**Committed 2026-09-15 before `product csharp regions` first ran on A.** Against the criterion in
`gateB-criterion.md`. No re-roll.

## §12.1 — reachable-undeclared

The 22 accepted entry points are the basket pages, the user/role/catalog endpoints, the
cookie logout and current-user actions, the register and confirm-email pages, the basket view
component and the JWT authenticate endpoint. Their Gate A paths were 1–25 production types
each and overlap heavily (identity, catalog, basket); the long Checkout path (49) is not
accepted. A has about 320 production types, 48 of them the Blazor client no accepted root
reaches.

| | Band | Centre |
|---|---|---|
| reachable-undeclared share of undeclared production types | **18–32%** | **24%** |
| error bound (unscored / edges of the union walk) | **2–6%** | 4% |

**Stated falsifier:** above 40% means the union walk pulls in far more than the per-candidate
paths summed, which would say the paths were not the union's parts.

§12.1 reads: the split discriminates — the bound is far smaller than the distance to 90.

## §12.2 — do the regions separate?

Of the 46 undeclared derived entry points:

| Region | Band | Centre |
|---|---|---|
| no facts under P-EP-4 | 28–36 | 32 |
| declarable | 5–12 | 8 |
| unstructured | 2–8 | 4 |

**Prediction on the clause:** declarable and unstructured **do** separate mechanically — no
per-symbol judgement is needed — but the separation covers a minority: the largest region will
be *no facts under the proxy*, because 48 of 68 candidates showed no fact at Gate A. The
finding to expect is not "the regions blur" but "the proxy is blind to most of A".

Spanning types: 2–6, most likely `OrderService` (three write-capable repositories, only the
basket act among the accepted covers part of them) and `BasketViewModelService`.
