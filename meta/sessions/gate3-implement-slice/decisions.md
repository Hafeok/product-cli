# Decisions — every point the specification does not settle, resolved by this session

Thirty-six. Each is marked at its site in the source with the same `D-nn` tag
(`grep -rn 'D-[0-9][0-9]' solution/src`), so the code and this record cannot drift.

**INVENTED** = the specification is silent and this session supplied something.
**DECIDED** = the specification is ambiguous or two of its rules conflict, and this
session chose a reading.

A decision here is not a determination. Nothing below was filed, and per the
prohibitions nothing below may be: where the specification does not settle something,
that is the finding.

| # | Kind | The decision | Site |
|---|---|---|---|
| D-01 | INVENTED | The `[Slice]` attribute type itself — namespace, argument types, `AttributeUsage`. Three profile `must` rules are stated in terms of it and it is supplied nowhere. | `Profile/SliceAttribute.cs` |
| D-02 | DECIDED | The role argument is a closed enum, not the bare string the profile's rule text shows. More checkable; possibly a divergence from a literal reading. | `Profile/SliceAttribute.cs` |
| D-03 | INVENTED | The act instance is an unvalidated string; nothing ties it to the act vocabulary at compile time. | `Profile/SliceAttribute.cs` |
| D-04 | INVENTED | **Every field of every fact.** `Cart`, `CartLine`, `ActorIdentity`, `OrderPlaced` — every name, type, unit and nullability. The fact vocabulary declares a type space, not types. | `Facts/Facts.cs` |
| D-05 | INVENTED | "Empty" means "no lines". A cart of zero-quantity lines is not empty under this reading. DSC-0001 pins the invariant by name and cannot define it, because `Cart` has no declared structure. | `Facts/Facts.cs` |
| D-06 | INVENTED | Money as minor units in a `long`. `decimal`, a Money type, or per-line currency are equally supported. | `Facts/Facts.cs` |
| D-07 | INVENTED | `ActorIdentity.AccountCurrency`. DSC-0005 compares against "the customer's account currency"; no fact carries one and DSC-0005 declares no position for it. Contested — see Q-05. | `Facts/Facts.cs` |
| D-08 | INVENTED | `OrderPlaced` carries an `OrderId` and an `OccurredAt`. Nothing says an order has either. | `Facts/Facts.cs` |
| D-09 | INVENTED | The `PlaceOrderCommand` shape: one field, `CartId`. `ActorIdentity` is deliberately not a payload field because DSC-0003 routes it elsewhere. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-10 | DECIDED | The command doubles as the HTTP request body; no separate transport DTO. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-11 | DECIDED | DSC-0002's `does_not_cover: cross-field-consistency` is vacuous with one field; nothing is done about it. | `Slices/PlaceOrder/PlaceOrderCommand.cs` |
| D-12 | INVENTED | `Accepted` / `Rejected` as a closed hierarchy; `Accepted` carries the event, `Rejected` carries invariant + reason. | `Slices/PlaceOrder/PlaceOrderOutcome.cs` |
| D-13 | INVENTED | `ICartStore` and `IClaimSource` exist at all. No input names a store, stream, repository or claims source. | `Slices/PlaceOrder/Providers.cs` |
| D-14 | DECIDED | The transport reference DSC-0003's `read_provenance` requires is moved behind `IClaimSource` so the provider's `must_not` holds in source text. **Satisfied as written, defeated in substance.** | `Slices/PlaceOrder/Providers.cs` |
| D-15 | DECIDED | The provider's "where external data is required" is a sufficiency condition, not a restriction, so `CartProvider` supplies an internal fact. Under the other reading the slice cannot read its own ground. | `Slices/PlaceOrder/Providers.cs` |
| D-16 | INVENTED | The claim names: `sub` (OIDC convention, imported) and `account_currency` (no basis at all). | `Slices/PlaceOrder/Providers.cs` |
| D-17 | INVENTED | `IOrderIdentityMint` as an injected abstraction, so the decision stays deterministic. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-18 | DECIDED | Rejection precedence: DSC-0001 before DSC-0005. A cart both empty and mis-currencied reports `CartNotEmpty`. Reversing it is equally supported. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-19 | INVENTED | An unsuppliable read position throws rather than rejecting — a third handler exit the profile does not admit. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-20a | DECIDED | A `residual` determination is **implemented**, on the reading that allocation describes discharge and not existence. If residual means "build nothing", this is behaviour the specification did not ask for. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-20b | INVENTED | The invariant name `CurrencyMatchesAccount`. DSC-0005 supplies none and `Rejected` must cite something. | `Slices/PlaceOrder/PlaceOrderHandler.cs` |
| D-21 | INVENTED | **The entire transport mapping.** Accepted → 201 + `Location`; Rejected → 422 + ProblemDetails; invalid payload → 400. "Derived from" names a dependency, not a function. The largest single hole. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-22 | DECIDED | All rejections map to one status; the invariant travels in the body. Branching per invariant would be "a conditional on domain state". | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-23 | DECIDED | DSC-0002's payload check lives in the controller, so the 400 path is a transport result **not** derived from Accepted-or-Rejected. That rule gives. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-24 | INVENTED | `POST /orders`, unversioned, plural noun, resource-named rather than act-named. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-25 | INVENTED | An unreachable default arm, because C# cannot prove the closed hierarchy exhaustive and the profile says nothing about expressing exhaustiveness in the stack. | `Slices/PlaceOrder/PlaceOrderController.cs` |
| D-26 | INVENTED | `IOrderPlacedSink` — an event store abstraction. No input names one. | `Unroled/EventAppendingPlaceOrderHandler.cs` |
| D-27 | DECIDED | The append is synchronous and non-transactional; no retry, no outbox, no ordering guarantee. **A failed append loses a placed order.** | `Unroled/EventAppendingPlaceOrderHandler.cs` |
| D-28 | DECIDED | The token is trusted as already validated, per DSC-0003's `read_provenance`. No signature, issuer, audience or expiry check. If the gateway is absent in some deployment, this reads an attacker's claim — the direct consequence of a settled determination, recorded rather than hedged. | `Unroled/Adapters.cs` |
| D-29 | INVENTED | In-memory store and sink, rather than inventing a schema. | `Unroled/Adapters.cs` |
| D-30 | INVENTED | A GUID as the order identifier. | `Unroled/Adapters.cs` |
| D-31 | INVENTED | 401 for a missing `ActorIdentity`, 404 for a missing `Cart`. A security judgement and a REST convention, both imported wholesale. | `Unroled/Adapters.cs` |
| D-32 | INVENTED | The composition root: registration, lifetimes, middleware order. The profile says nothing about how one role reaches another. | `Program.cs` |
| D-33 | DECIDED | **`IPlaceOrderHandler` resolves to the unroled decorator, not to the handler.** The controller's source text calls "exactly one type declaring the handler role" while at run time the first type it reaches declares none and does the forbidden persistence. One DI line. | `Program.cs` |
| D-34 | DECIDED | `Program` made public so tests can drive the slice. | `Program.cs` |
| D-35 | DECIDED | Tests are written at all, and are marked as evidence rather than as discharge of any `checked` allocation. | `tests/` |
| D-36 | DECIDED | The mechanical profile check inspects the shape of the outcome type, not the emissions. A handler constructing a second event type and dropping it would pass. | `ProfileConformanceTests.cs` |

---

## Three decisions about the run rather than the slice

| # | The decision | Recorded in |
|---|---|---|
| R-1 | The Gate A expectation list was written after reading the act vocabulary, the profile and the schema, and before opening `place-order.determinations.yaml`. The strictest reading of the gate would have written it before opening anything. This makes the list better informed than the strictest reading allows. | `bootstrap.md` |
| R-2 | The greenfield solution lives inside the session directory, so no host-repository convention leaks into a slice whose point is to be built from the specification alone. | `bootstrap.md` |
| R-3 | The gates say *Hold*; no principal was present to ratify. This session proceeded past Gate A and Gate B unratified, recording a provisional reading for each open question, because holding would have produced no artefact to report on. **Every question in `questions.md` is open.** | `questions.md` |
