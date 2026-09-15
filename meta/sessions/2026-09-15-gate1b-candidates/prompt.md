# Session: Gate 1b — entry-point candidates and the delta

**Commit this prompt and a bootstrap record to `meta/sessions/` as the first act. Session-neutral commit identity: `Claude <noreply@anthropic.com>`.**

---

## What changed

Gate 1b was blocked on an act vocabulary nobody could author without fitting it to the code. CG-R-105 unblocks it: **the vocabulary is derived from external integration points**, which are mechanical and need no domain modelling.

It is a **measurement vocabulary** — transport-shaped, good for computing the delta, not good for accruing determinations against. Every figure it produces is labelled transport-derived.

Read `rulings-cg-r-105-110.md` before anything else. It is the instruction set for this gate and the prohibitions below follow from it.

---

## Prohibitions

- **Do not name acts.** A candidate carries the transport name, the path and the observed positions. It does not carry an act name, what the act settles, or who answers. Naming is ratification and it is not yours (CG-R-106).
- **Do not generate slice declarations.** R-D stands: the candidate set is a prompt, never a queue for approval.
- **Do not infer expected actors or scale.** Observed authorisation evidence is read from code and labelled observed. Everything else is an **unfilled slot**, and an unfilled slot is *not stated*, never *no actors* (CG-R-108).
- **Do not author the invited determination.** Supported throughput is a determination someone settles after acceptance, not a slot the session fills (CG-R-109). Carry the field; leave it empty.
- **Do not treat a library's public surface as entry points.** Libraries get boundary declaration over the used surface, not act derivation.
- Do not proceed past a gate without ratification.

---

## Gate A — the candidate set

Derive candidates from A's external integration points: controller actions, Razor page handlers, ViewComponents, and any subscription or scheduled trigger.

**Each candidate carries:**

| Field | Source |
|---|---|
| transport name and path | observed |
| HTTP method or trigger kind | observed |
| the path from it — the types reached, per the walk | observed |
| facts read and written along that path | observed |
| authorisation attributes, roles, auth scheme, anonymous access | **observed, labelled** |
| any client-identity check on the path | **observed, labelled** |
| expected actor kinds | **unfilled ground slot** |
| population, order of magnitude | **unfilled ground slot** |
| rate, order of magnitude | **unfilled ground slot** |
| supported throughput — sustained, peak, window, behaviour above the limit | **invited determination** (CG-R-109) |

Report the count and, separately, **the candidates whose paths overlap** — two entry points reaching the same types. Those are the merge candidates and they are the signal (CG-R-106).

**Razor pages are in scope as entry points** even though `@inject` edges are not readable. A page that renders is an integration point whether or not its injections are visible; state the blind spot beside it per CG-R-78.

**Hold.**

---

## Gate B — the delta

With the candidate set ratified and slices declared against whichever candidates were accepted, compute the three regions:

- **declared** — an entry point with a declared slice
- **declarable** — no declared slice, and a clean path from the entry point
- **unstructured** — a path where the act boundary runs through a type, per CG-R-52's separator with its incidence stated

And the two ratios: **reachable-undeclared** and **isolated-undeclared**, reported separately, never averaged.

**With the error bound beside them** (CG-R-89). Every unscored composition edge could reclassify a type between reachable and isolated; the unscored fraction is the bound, and a split whose bound exceeds the difference it reveals does not discriminate.

**Every figure labelled transport-derived** (CG-R-105).

**Hold.**

---

## Gate C — report

1. The candidate set, with the merge candidates called out.
2. The three regions by cluster, never a flat list of symbols.
3. Both ratios with the error bound.
4. **§12.1's disposition** — the first time it can be read against the split it actually asks about, rather than against a proxy.
5. What this does not establish: the vocabulary is transport-derived and no domain modeller has seen it; the actor and scale slots are unfilled; A is a maintained reference implementation and not the brownfield codebase the delta was designed for.
6. Weakest point.

**Stop.**

---

## Carried

- The instrument is validated and closed to further work per CG-R-103, excepting CG-R-101's table ground truth. **Do not improve the reader during this gate.** A defect found is reported, not repaired — repairing it would move the denominator mid-gate for the seventh time.
- `channel` is a named extent axis from now on (CG-R-107), which matters when determinations are eventually written against accepted candidates, not during this gate.
- Solution B is not in scope. One solution, one gate.
