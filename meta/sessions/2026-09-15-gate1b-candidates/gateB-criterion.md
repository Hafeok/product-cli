# Gate B — the delta criterion, stated before it runs

**Status: `[PROPOSED]`, 2026-09-15.** Committed before the regions are computed (CG-rule-08).
Every figure Gate B produces is **transport-derived** (CG-R-105) and **stand-in** (CG-R-115): the
vocabulary is Emil's ratification of a transport-shaped candidate set for a codebase nobody holds
intent for.

## 0. The deferred row, established (CG-R-114)

Emil deferred `fastendpoints:UpdateRoleEndpoint@UserManagementEndpoints.(type)#PUT` because the
candidate is named `UpdateRoleEndpoint` and the route reading came from
`UpdateUserEndpoint.cs`. Established from the source and the inventory: the file
`src/PublicApi/UserManagementEndpoints/UpdateUserEndpoint.cs` declares, at line 13,
`public class UpdateRoleEndpoint(UserManager<ApplicationUser> userManager) : Endpoint<UpdateUserRequest, …>`
and at line 17 `Put("api/users")`. The inventory records the type
`T:Microsoft.eShopWeb.PublicApi.UserManagementEndpoints.UpdateRoleEndpoint` at that file and
line. **Neither the candidate id nor the route attribution is wrong: the source's class name
disagrees with its file name.** A second, unrelated `UpdateRoleEndpoint` lives in
`RoleManagementEndpoints/UpdateRoleEndpoint.cs` (`Put("api/roles")`), which is why the ids carry
the namespace tail. The deferral stands — it is Emil's — and the row is undeclared for Gate B.
The disagreement is A's, reported as **F-EP-3**.

## 1. The vocabulary, as ratified

From `ratification-A-filled.yaml`, rows with `decision: accept`: 22 entry points, 20 distinct
acts (`EndSession` over two entry points, `ReadUserDetails` over two). An act's **positions**
are read from its entry points' paths through proxy P-EP-4, stated:

- `reads(a)` = ∪ over the act's entry points of `facts.read` ∪ `facts.dbset_touched`;
- `writes(a)` = ∪ of `facts.written` ∪ `facts.possibly_written` (a superset: the call site's
  type argument is not in the inventory, so a possible write counts as a write).

An act's `channel` is carried from the worksheet (CG-R-107). Rows with `decision` `reject`,
`defer` or unset are **undeclared** entry points. The three hand-supplied rows have no path
and are reported on their own line, outside the regions.

## 2. The separator — CG-R-52's proxy on P-EP-4 facts

For every production type `T` on any candidate's path, `F(T)` is the fact set P-EP-4 reads on
`T` alone: the type arguments of `T`'s repository-typed constructor parameters and the `DbSet`
properties `T`'s members access. `T` is **spanning** when `F(T)` is non-empty and no act's
`reads ∪ writes` covers it (CG-R-52, applied per type as `csharp_delta` states it).

For an undeclared entry point `E` with path `P(E)`, `F(E)` = ∪ `F(T)` over `P(E)`:

| Region | Rule |
|---|---|
| **declared** | `E` is accepted (it carries an act) |
| **declarable** | `F(E)` non-empty, no spanning type on `P(E)`, and some single act's positions cover `F(E)` — listed as declarable *for those acts* |
| **unstructured** | `F(E)` non-empty and either a spanning type lies on `P(E)` or no single act covers `F(E)` (the act boundary runs through the path); the reason is printed |
| **no facts under the proxy** | `F(E)` empty — P-EP-4 reads nothing on the path (identity state lives behind `UserManager`/`RoleManager`, which the proxy does not read); **its own row, never folded** (CG-R-62) |

Reported by kind and by host — never a flat list of symbols (R-D). Incidence of the separator
on A is printed: spanning types / types with facts.

## 3. The ratios and the error bound

Over the production types that are not an accepted entry point's own type: **reachable-
undeclared** (in the walk from the union of the accepted entry points' roots, through the
union of the hosts' registration sites, O-17 off — the same walk as the candidate paths),
**unresolved-undeclared** (implementors of an unresolved or partial edge's target, never
folded), **isolated-undeclared** (the rest). By namespace and by project.

**Error bound (CG-R-89):** unscored edges of that walk / its composition edges. Every unscored
edge could move a type between reachable and isolated.

## 4. §12.1, read against the split it asks about

§12.1: "if more than ~90% of types are reachable from some declared entry point, the
reachable-undeclared ratio does not discriminate." Reported: the reachable share, the bound,
the distance to 90, and whether the bound is smaller than that distance. No fire/clear form
(CG-R-89); the statement is printed.

## 5. What every figure carries

- transport-derived (CG-R-105); stand-in (CG-R-115);
- P-EP-4's incidence: unmeasured; the positions are a superset on the write side;
- the table (`csharp_di_knowledge`) is **unvalidated** — CG-R-101's ground truth is
  outstanding; registration-not-read verdicts on the paths rest on it;
- L-EP-1 … L-EP-4 as printed by the candidate derivation; F-EP-1 (primary constructors) does
  not affect composition edges.

## 6. Predictions

The session's is `prediction-session-gateB.md`, committed with this criterion. Emil's has not
been requested for Gate B; the measurement is built and its run held for it, or for an explicit
waiver — CG-R-59's two-prediction design has governed every measurement so far.

---

## Appended — CG-R-120, before the run

**§3 is not amended.** CG-R-120 rules the bound's form: the error bound is computed over every
edge the walk could not follow — `unresolved` and `registration-not-read` together — and printed
as one bound with its composition beneath. The instrument prints that bound as the bound in
force, and the CG-R-89 unscored fraction (§3 as written) beside it, labelled as the prior form,
because the session's prediction was committed against it. Each prediction is scored against
its own form. §12.1's statement uses the CG-R-120 bound.
