# Invocation — measurement rulings and the denominator criterion, verbatim

Received 2026-09-14 with `rulings-cg-r-63-67.md`, filed beside it as `rulings-gate1a-measurement.md`.

---

# Reply to the C# binding session — measurement rulings and the denominator criterion

**Attach:** `rulings-cg-r-63-67.md`

---

## The message

Rulings attached. File verbatim, then work them in the order below. Gate 1b remains blocked.

### 1. What the rulings settle

**CG-R-63.** Both predictions were wrong in the same direction by the same amount, which is one shared assumption rather than two errors. Neither was about the measured quantity: *the fraction of interface-mediated edges the resolver can follow* was never operationally defined, so the denominator admitted things nobody intended. Any measure whose prediction is to be scored has its denominator defined before the prediction is committed.

**CG-R-64 (O-9).** Apply the narrowing. It survives the defended-from-the-definition test without citing the number it produces: an abstract base class used as data has no implementations to choose between, so there is nothing to resolve. Defect repair, not tuning. **A's prediction comparison is void** — both predictions were made against the old denominator. Report both numbers with their definitions, mark A **unscored**, and do not reconstruct what anyone would have predicted for 78%.

**CG-R-65 (O-10).** Module `Startup` classes are part of the primary convention, following from CG-R-60 rather than as a new choice: they *are* the registration mechanism, and excluding them is refusing to read the registrations. **B's primary is 53.0%**; retain 40.3% labelled *registrations not read*.

**CG-R-66 (O-11) is blocking.** An edge neither followed nor reported unresolved is invisible failure, and worse than either category. Gate 1b does not begin until the cause is established, and until then **no isolated count from either solution is trusted** — nothing distinguishes three handlers from a pattern. If the cause proves unfixable, the outcome is a fourth reported category, *not followed, cause unknown*, not leaving those types in *isolated*.

**CG-R-67.** The ignore globs are ratified; the silence is not. A check that cannot run reports *skipped, host unavailable*, and a run containing a skip is not reported as fully green. This is the third instance of one class — `run_all.sh` green on a crashed check, gate runs grepping past a failure, now a glob — and the same fix is owed to `run_all.sh` in the binding.

**§12.1 is not ruled fired.** A's firing may be a denominator artefact; B's at 53.0% looks genuine. The disposition is read after the criterion below is applied and O-11's cause is established.

### 2. The denominator criterion — the next work

The measure cannot go further until it is defined what an interface is *for* at a given edge. One criterion, not a taxonomy:

> **An edge enters the denominator if and only if the dependency must be satisfied by a chosen implementation at composition time.**

It is a **per-edge** test, not per-interface. The same interface may be injected at one site and type-tested at another; only the first is resolution.

The roles follow from the criterion rather than being invented beside it:

| Role | In denominator | Why |
|---|---|---|
| service abstraction | **yes** | an implementation is chosen at composition |
| generic dispatch contract | **yes**, resolved by declaration | the type parameter is the act (CG-R-61) |
| factory / provider abstraction | **partial** | composition chooses the factory; the factory chooses later |
| marker | no | no member to satisfy |
| data contract | no | a shape, not a dependency |
| capability, tested for | no | a type test chooses nothing |
| abstract base used as data | no | nothing to choose between — O-9 |

**Two requirements on the implementation.**

**Recognition rules are proxies and are declared as such.** *No members implies marker*; *properties only implies data contract*. Each is mechanically checkable and each will misclassify something. The criterion is the original predicate, the shape rule is the proxy, and the divergence is stated per rule. Otherwise an undefined denominator has been replaced by a confidently wrong one.

**The factory row is probably where Orchard's residue lives.** The roughly seven hundred registration-wrapper calls are composition choosing something that will choose again later. That is genuinely partial resolution; forcing it into resolved or unresolved misreports it either way. Report partial as its own state.

### 3. Order

1. O-11's cause. It blocks everything and its extent is unknown.
2. The criterion and the per-edge role classification, with the proxy divergences stated.
3. Recompute both solutions. **Neither prediction survives the recomputation** — CG-R-64 voids A's and the criterion voids B's by the same argument. Report the old and new figures side by side with the definitions that produced each.
4. Then §12.1's disposition, on the recomputed figures.
5. Then the `run_all.sh` skip-reporting fix, per CG-R-67.

### 4. Predictions

**None are committed now.** Per CG-R-63, a prediction is committed only after the denominator is defined and ratified. When the criterion is settled, propose it and hold — both predictions are made against the criterion, before the recomputation runs.

### 5. Also outstanding

O-12 was not described in what reached me and is unruled. State it again.

The act vocabulary for Gate 1b is mine and remains outstanding. Do not author one.

Hold.
