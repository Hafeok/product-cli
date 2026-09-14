# Rulings CG-R-68 … CG-R-70 — the denominator criterion

**Issued by Emil, 2026-09-13.**

---

## CG-R-68 — The criterion is ratified, with one amendment

**Ratified as proposed:** composition edge as a constructor parameter of a container-constructed type or a service-locator argument; the seven role proxies printing with their divergences; factory and provider targets held as *partial* and never divided; the fixture exercising every row.

**One amendment, and it follows from the library-boundary reading.**

An edge to an external abstraction with **no in-solution implementation** — `ILogger<T>`, `IHttpClientFactory`, a framework or package contract satisfied by the framework itself — is neither resolved nor unresolved. It is a **boundary** edge.

Counting these as *unresolved* would tank coverage over edges no resolver should be expected to follow, and would report the instrument's correct behaviour as its failure. Counting them as *resolved* would hide the undeclared library boundary, which is the thing the mapping exercise exists to surface.

**Ruled: a fourth state, `boundary`, reported separately and excluded from the resolved/unresolved denominator.** Each records the external type and the assembly. That set is the used library surface, and it is what needs declaring at member level — `SaveChanges` is a terminal verdict with transactional semantics; "we use EF Core" settles nothing.

**Three points of operational form to state in the record:**

- *Container-constructed* is determined from the registration list, not inferred from shape.
- **Property and method injection** are composition edges by the criterion — a dependency satisfied by a chosen implementation at composition time — and are rare in these solutions. State whether the reader emits them; if it does not, that is a stated gap, not an absence.
- **CG-R-61's declaration shortcut does not apply here.** Neither solution carries declarations, so generic dispatch contracts fall back to resolution, and MediatR's assembly-scanning registration is the hard case rather than the easy one.

---

## CG-R-69 — O-11's fix is the third denominator change. Nothing measured before it is comparable.

Edges to targets declared outside the solution were never emitted. So every interface-mediated edge to a package abstraction — precisely the hardest class to resolve — was **absent from the denominator entirely**, in every run so far.

**Run 4's figures are over a defective denominator.** 85.7% for A and 62.3% for B are not near-final and must not be read as approaching a result. The missing class is the one that would lower them.

**Ruled:**

- Runs 3 and 4 are retained as **instrument history, not measurements**. Their numbers are not compared with anything computed after the fix.
- The recomputation runs against the criterion **with the O-11 fix applied**, so the external-abstraction edges are in the denominator or in `boundary`, and neither silently absent.
- **This is the third time a denominator moved after numbers existed.** CG-R-63 required the denominator to be defined before a prediction is scored. It is now also required that the *reader's emission rules* are fixed and stated before a prediction is committed — a denominator is not defined if the instrument silently omits a class of its members.

**A small correction to my own record.** CG-R-64 cited about 78% for A from a hand estimate off a truncated list; the computed figure was 85.7%. Eight points wrong. Hand estimates from partial output do not enter rulings again.

---

## CG-R-70 — Prediction, committed against the ratified criterion

Against the criterion as amended at CG-R-68, with the O-11 fix applied and `boundary` excluded from the denominator:

| Solution | Resolution coverage | Centre |
|---|---|---|
| **A — eShopOnWeb** | 88–96% | **92%** |
| **B — Orchard Core** | 58–72% | **65%** |

**Reasoning, so the prediction can be wrong for a stated reason.** Removing framework abstractions to `boundary` and excluding markers, data contracts and capability tests leaves A as mostly explicit `AddScoped<IFoo, Foo>()`, which resolves. The residue is MediatR's scanning registration, a handful of edges. B keeps module registration read as roots, gains the external-abstraction class in the denominator, and pushes the wrapper calls into `partial` — the first lowers it, the third does not count against it.

**Expected unresolved distribution.** A: assembly scanning almost exclusively. B: module configuration first, assembly scanning second, keyed and decorator minor.

**Centres are given because a thirty-point band spanning the §12.1 line cannot be wrong**, which was the objection made to an earlier prediction and applies equally here.

---

## Also

- **CG-R-67 is discharged for `ddd diff-contracts`.** The `run_all.sh` patch proposal is accepted in principle; it applies against the binding's repository, which is mine to attach.
- O-12 is restated in the O-11 record and is read there.
- Both v3 inventories exist and are unwalked. They are walked after the predictions are committed, not before.
- Register debt: CG-R-17 … CG-R-70. Mine.
