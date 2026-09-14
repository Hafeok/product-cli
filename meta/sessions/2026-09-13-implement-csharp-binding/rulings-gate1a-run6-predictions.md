# Rulings CG-R-81 … CG-R-84 — run 6 predictions

**Issued by Emil, 2026-09-14.** The restatement surfacing six unstated rules, two of which were live defects on the fixture, is CG-R-80 working as intended. The state order — provider, value, library-provided, role, resolution — being a written rule rather than an emergent property of check sequence is the repair that matters most.

---

## CG-R-81 — Prediction committed

Against the rules as restated, with P-1 to P-7 applied and reader v5:

| Solution | Band | Centre |
|---|---|---|
| **A — eShopOnWeb** | 87–94% | **91%** |
| **B — Orchard Core** | 68–80% | **74%** |

**Reasoning, so it can be wrong for a stated reason.**

*A.* The marker repair returns roughly nineteen edges to the denominator, dominated by `IRepository<T>`, which eShopOnWeb registers explicitly and which should resolve. Thirty-two of the thirty-six unresolved move to `registration-not-read`. What remains unresolved is small: one no-registration and three conditional.

*B.* Retiring the factory-provider-by-return-type reading moves 231 edges out of `partial` and into roles that must now actually resolve, and retiring data-contract returns 181 more. Both enter a denominator they were previously outside. Excluding test projects removes 86 of the 130 unresolved, which helps. My expectation is that the newly-scored edges resolve at a materially lower rate than the previously-scored ones, because they were classified out precisely by shapes that were hard to follow.

**My B centre sits at 74%, one point under the §12.1 line, deliberately.** It says B is borderline and that the ratio's disposition turns on the repairs rather than on the codebase. If B comes in above 80% I was wrong about how much the retired readings were hiding.

---

## CG-R-82 — A band spanning the disposition line is recorded as not committing

Emil's prediction is A 80–100%, B 70–95%. Committed as given.

**B's band spans §12.1's line at 75%**, so no outcome inside it can be wrong about the disposition. A's does not — every value in 80–100 clears.

**Ruled: this is recorded on the prediction, not corrected.** A prediction is the predictor's to make and a band is a legitimate form. What is not legitimate is discovering afterwards that nobody had committed to a side and then arguing about what was meant, which CG-R-59's two-prediction design exists to prevent.

So the record states: on §12.1's disposition for B, the session's prediction commits (clears), mine commits (fires, narrowly), and Emil's does not.

---

## CG-R-83 — `registration-not-read` stays in the denominator, and the incentive is named

The rule is not stated in what reached me and it decides A's headline by a wide margin.

**Boundary is legitimately outside the denominator**: nothing is to be resolved, because no choice is made at composition. `registration-not-read` is different in kind — it is **the instrument's ignorance of a registration that exists**. The edge is resolvable; the reader cannot read the call.

**Excluding it means coverage rises as the table's ignorance grows.** A reader that parsed nothing would have no unresolved edges and a perfect score. That is the flattery direction CG-R-62 named, in the state that was created to prevent it.

**Ruled:**

- `registration-not-read` is **inside** the denominator and counts against coverage, reported as its own subset so the cause is visible.
- **Do not change the denominator after predictions are committed.** If the restatement placed it outside, the predictions stand against that rule and **both figures are reported** — coverage with it in, coverage with it out, labelled. This ruling governs from run 7.
- The restatement states its membership explicitly. A denominator rule that has to be inferred from a table column is not a stated rule, which is CG-R-63 in a fifth form.

---

## CG-R-84 — `ViewComponent` is a root in A

Framework-activated, exactly as controllers and Razor Pages are, and the root convention already admits framework activation conventions. **Ruled: included**, and its two known edges leave the recall miss list.

Two edges is within the noise of every band committed, so it does not disturb the predictions and does not need to wait for run 7.

---

## Also

**The mistyped commit message** — A's band as 84–96 rather than 84–94 — with the file as the record and the error noted in it, is the correct disposition. A commit message is immutable and cannot be corrected in place, so the correction lives where the record lives. Consistent with CG-R-48's distinction between a record and a presentation surface.

**The fixture's table coverage of 13 parsed, 3 known, 0 unknown of 16** is the shape wanted. The figure that matters is the same three numbers on A and B, where the unknown column will not be zero.

**Razor:** 45 directives in 71 files for A, 342 in 1,610 for B. Both print beside their headline per CG-R-78. A's 45 is larger relative to A's 91 composition edges than B's 342 is to B's 1,450, which is worth noticing — A's blind spot is proportionally the worse of the two.

---

Gate 1b's act vocabulary remains mine. Register debt: CG-R-17 … CG-R-84.
