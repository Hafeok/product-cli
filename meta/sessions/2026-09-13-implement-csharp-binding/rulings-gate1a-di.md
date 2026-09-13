# Rulings CG-R-60 … CG-R-62 — DI resolution and the reachability measure

**Issued by Emil, 2026-09-13.** CG-R-60 **supersedes CG-R-58**. CG-R-59's prediction table is withdrawn and replaced at CG-R-62. CG-R-57 is unaffected.

---

## CG-R-60 — There is one graph. DI-off is not a setting.

CG-R-58 treated DI traversal as a parameter and ruled which setting the threshold should be read against. That was wrong.

**In a solution built with DI, the registrations are the call graph.** A graph built without resolving them is not a conservative view of the program — it is a graph of a different program, one that does not exist. Reporting a percentage from it is not a lower bound; it measures nothing.

Worse, offering it as a setting invites the abuse CG-R-58 was written to prevent: pick the flattering number, keep the ratio.

**Ruled:**

- **A solution built with DI is never loaded without resolving it.** There is no DI-off number, and none is reported.
- **CG-R-58 is superseded.** The question was never which setting to judge against. There is one graph, and the open question is whether it can be built.
- Root convention remains a real choice and is still reported with what it includes and excludes. That is a choice about where to start, not about whether the edges exist.

---

## CG-R-61 — Declaration supplies the edges. Resolution is only needed for undeclared code.

Convention dispatch over open generics — `IRequestHandler<PlaceOrder, Result>` — resolves by the act itself: the type parameter *is* the act instance. So `[Slice("PlaceOrder", "handler")]` states the edge directly, and the resolver never needs to understand the registration.

**That is the case which breaks a syntactic graph, and the declaration dissolves it rather than solving it.**

**Ruled:**

1. **Declared code needs no DI resolution.** Where both ends carry declarations, the edge is read from the declarations.
2. **Undeclared code reached through an interface still needs resolution.** That is exactly the reachable-undeclared measure, which is the one ratio that mattered.
3. **Resolution coverage is reported as a first-class figure** — what fraction of interface-mediated edges the resolver could resolve. It is the number that decides whether the instrument is viable, and it is wanted before any reachability percentage.

**Three cases are not actor variation, and must not be assumed to resolve by declaration.** The claim that multiple implementations always indicate a changing actor does not hold generally:

| Case | Why it is not actor variation |
|---|---|
| **decorators** — logging, caching, retry wrapping one interface | same actor, same act, same call site; the chain adds cross-cutting behaviour |
| **data-driven strategy** — tax by country, pricing by tier | the caller is identical; the selector is a domain value, so this is a determination *about* the act |
| **module enablement** — which implementation is live depends on deployment configuration | Orchard Core's entire model |

Environment swaps and test doubles do bend toward the actor reading — a stub gateway is a different external party, a test harness is a different actor performing a different act — and the resolver may treat them so. The three above may not.

---

## CG-R-62 — Unresolved is its own category, and the bias runs toward flattery

An edge the resolver cannot follow makes an undeclared type look **isolated** rather than **reachable**. Isolated-undeclared is honest untouched work; reachable-undeclared is a specification that looks complete and is not. So an unresolved edge moves a type from the urgent category to the benign one, and **the instrument's blindness makes the codebase look better specified than it is.**

That is the one direction an assessment tool must not fail in.

**Ruled:**

- **Unresolved is reported as its own category**, never folded into unreachable or isolated. Conflating them presents the instrument's blindness as a finding about the codebase — the same defect as a proxy recorded without its divergence.
- Each unresolved edge records **why**: assembly scanning, keyed service, decorator chain, conditional registration, open generic, or module configuration. The distribution of reasons is the map of what the resolver would have to learn next.
- **No reachability figure is reported without its unresolved count beside it.**

**§12.1 is restated.** It is no longer a threshold on a reachability percentage judged across settings that no longer exist. It fires when, on a real DI-built solution, **resolution coverage is low enough that the reachable/isolated split cannot be trusted** — that is, when the unresolved count is large relative to the undeclared population it would reclassify.

The precise form is fixed at the measurement, with the prediction at CG-R-62 committed first.

---

## Prediction, replacing CG-R-59's table

**Recorded expectation, before measurement:**

| Solution | Resolution coverage | Basis |
|---|---|---|
| eShopOnWeb | **85–95%** | mostly explicit `AddScoped<IFoo, Foo>()`; the MediatR handlers are the open-generic case and resolve by declaration under CG-R-61 |
| Orchard Core | **40–65%** | module registration, conditional enablement and assembly scanning dominate; this is the hard end |

**Consequence if it holds:** the instrument is viable on conventionally-wired solutions and not on module-registered ones, and §12.1 fires on Orchard Core and clears on eShopOnWeb. That is a finding about which shape of codebase the tool serves, not a verdict on the tool.

Stated as numbers rather than as ranges wide enough to be unfalsifiable.

---

Register debt: CG-R-17 … CG-R-62. Mine.
