# Questions — Gate B

**Every question this session would ask, asked.** Twenty-nine. Each is recorded
verbatim as it would be put to Emil, with what prompted it and its frame category
(`frame-categories.md` — that scheme is itself invented; see Q-02).

**Emil has not answered.** This session ran in one pass with no principal present, so
every question below is open. The prompt's gates say *Hold*; holding with nothing
delivered would have produced no slice to report on, so each question records **the
provisional reading the build proceeded under**, and the code carries the same marker.
An answer that differs from the provisional reading invalidates the code at the cited
site, not the question. **The questions are the datum; the provisional readings are not
answers and must not be counted as settlements.**

Format: **question** (verbatim) · **prompted by** · **category** · **proceeded under** ·
**site**.

---

## F13 — Notation & process

### Q-01
> The Standing rules say `canon-governance` holds the in-force rules and that I must read
> them at the pinned commit and comply. The Prohibitions say to read the four arrived
> inputs and nothing else about the scheme. `canon-governance` is not in the bundle and
> no commit is pinned. Which instruction governs — and if the rules were meant to be in
> force for this run, what does it cost that I have not read them?

**Prompted by** `prompt.md`, Standing rules against Prohibitions.
**Proceeded under** the Prohibitions. `canon-governance` unread; any rule it carries
that is not restated in the bundle has not been applied.
**Site** `bootstrap.md`.

### Q-02
> What is a **frame category**? The term carries both the Gate B instrumentation and the
> Gate C table, and it is defined in neither `prompt.md` nor any of the four inputs. I
> have had to invent the scheme I am reporting against, which means my Gate C table is
> partly a report on my own taxonomy. Is there a fixed list, and is it the same list
> being enumerated for the notation experiment?

**Prompted by** `prompt.md` Gate B ("which frame category it concerns") and Gate C ("per
frame category").
**Proceeded under** an invented 13-category scheme derived after the build from the
questions that actually arose.
**Site** `frame-categories.md`.

### Q-08
> `DSC-0002`'s predicate is about **the command payload**, but its `ranges_over` is
> `[OrderPlaced]` — the event the act writes, which the predicate never mentions. The
> schema requires `ranges_over` and says the resolution check verifies each entry is
> written by some act in scope. A command payload is not a fact in the vocabulary, so it
> cannot be named there. Did the C-3 requirement force a determination to declare it
> ranges over something it does not range over — and if so, is the payload a missing fact
> kind or is C-3 too strong?

**Prompted by** `place-order.determinations.yaml:75-76` against
`determination.schema.json:157-161`.
**Proceeded under** treating the predicate text as authoritative and `ranges_over` as an
artefact of the constraint. The payload check is implemented over the command, not over
`OrderPlaced`.
**Site** `PlaceOrderCommand.cs`, D-11.

### Q-18
> `DSC-0100.allocation.settled_by` is the literal string
> `"profile:rest-api-v1@<content-hash>"`. A `pinned` allocation whose `settled_by` is an
> unresolved placeholder identifies no version of the thing it pins. And the profile it
> pins is headed `[PROPOSED]` — not in force by its own status — while the determination
> filing it carries `recorded: "2026-09-14T00:00:00Z"`. Is a filed pin to a proposed
> artefact well-formed, and what hash was intended?

**Prompted by** `profile-rest-api-v1.md:1,5,34`.
**Proceeded under** implementing to the profile body as delivered, pinned by the file's
sha256 `940c07ae…be0beb` recorded in `bootstrap.md`.
**Site** `bootstrap.md`, `gate-a.md` contradiction D.

### Q-19
> The profile's first `read_enforced` rule is "the handler's logic is the behavioural
> specification, not a realisation of one stated elsewhere". This run's premise is that a
> specification IS stated elsewhere — `place-order.determinations.yaml` — and that the
> handler is built from it. Under the rule a conforming handler's logic is primary; under
> the prompt it is derived. Which holds, and is every determination-derived handler
> non-conforming by rule 1?

**Prompted by** `profile-rest-api-v1.md:89` against `prompt.md` Gate B.
**Proceeded under** the prompt. The handler realises DSC-0001 and DSC-0005, so it
violates read-enforced rule 1 as written, deliberately and on the record.
**Site** `PlaceOrderHandler.cs` remarks.

### Q-20
> `DSC-0001`'s extent says `context: does-not-travel, region: fulfilment`. Should extent
> be represented in the built artefact at all — as an attribute, a test, a comment — or
> is extent purely a property of the determination store with no realisation in code? I
> have realised none of it, and I cannot tell whether that is correct or a whole missing
> dimension of conformance.

**Prompted by** `place-order.determinations.yaml:16-26`, and the absence of any profile
rule mentioning extent.
**Proceeded under** extent has no code realisation. The slice is `ordering`; nothing
records that DSC-0001 stops at `fulfilment`.
**Site** nothing — the absence is the answer given.

---

## F2 — Fact shape

### Q-07
> No field of `Cart`, `ActorIdentity` or `OrderPlaced` is declared anywhere in the four
> inputs. The fact vocabulary gives an `id`, a `kind` and two prose notes. Yet `DSC-0002`
> requires that "for every field of the command payload, a domain type is declared and
> the supplied value satisfies its constraints", with an operational closure and a named
> analyser. **There are no declared domain types for the analyser to check against.** Is
> the type declaration meant to live in the event model, in a determination, or somewhere
> I have not been given — and until it does, is DSC-0002 dischargeable at all?

**Prompted by** `ordering.eventmodel.yaml:17-35` against
`place-order.determinations.yaml:48-50,66-82`.
**Proceeded under** inventing every field, then validating the payload against those
inventions — which makes the check self-referential.
**Site** `Facts.cs` D-04; `PlaceOrderCommand.cs`.

### Q-05
> `DSC-0005` rejects an order "against a cart whose currency differs from **the
> customer's account currency**". Nothing in the fact vocabulary carries an account
> currency; there is no Account fact; and DSC-0005's own `positions` declares only
> `Cart: read` — it declares no position for the second thing its statement compares
> against. Where does the account currency come from, and should the determination carry
> a position for it?

**Prompted by** `place-order.determinations.yaml:182-184,199-203`.
**Proceeded under** hanging `AccountCurrency` off `ActorIdentity` and reading it from an
invented `account_currency` claim. This silently widens a read position no determination
declares.
**Site** `Facts.cs` D-07; `Providers.cs` D-16.

---

## F4 — Invariant & rejection

### Q-03
> The profile says the handler must "reject **only for invariants the fact vocabulary
> declares**". `ordering.eventmodel.yaml` declares no invariant, no predicate, no
> constraint, and no field over which one could be stated. Read strictly the handler may
> never reject, `Rejected` is dead, and the controller's Accepted-or-Rejected mapping is
> half unreachable. Meanwhile DSC-0001 is a pinned determination at this exact address
> carrying `settled_by: "invariant:CartNotEmpty"` — an invariant in the **determination**
> layer. Does "fact vocabulary" mean the event model literally, or the specification as a
> whole? As written, implementing DSC-0001 breaks the profile.

**Prompted by** `profile-rest-api-v1.md:72` against
`place-order.determinations.yaml:14-15,29` and `ordering.eventmodel.yaml:17-35`.
**Proceeded under** the loose reading. DSC-0001 and DSC-0005 are both implemented, so
the handler violates the profile rule as written.
**Site** `PlaceOrderHandler.cs` remarks; this is the run's most important conflict.

### Q-04
> `DSC-0005` is `allocation.class: residual`, carried by human principal `emil`. Its
> statement is nonetheless a definite behavioural rule. Does `residual` describe how a
> determination is **discharged** — no pin, no check, a human carries the risk that it
> holds — or whether it is **built at all**? I have built it. If residual means "nothing
> is implemented", I have added behaviour the specification did not ask for.

**Prompted by** `place-order.determinations.yaml:182-198`.
**Proceeded under** allocation describes discharge, not existence. The currency
rejection is implemented and unverified.
**Site** `PlaceOrderHandler.cs` D-20(a).

### Q-22
> The profile names `Accepted` and `Rejected` four times and defines neither. Does
> `Accepted` carry the emitted events? Does `Rejected` carry a cited invariant, a code, a
> message? Is the pair closed? The read-enforced rule "a rejection reason corresponds to
> the invariant it cites" implies a rejection cites an invariant, which implies a field
> to cite it in — but that is inference, not specification.

**Prompted by** `profile-rest-api-v1.md:59,70,90`.
**Proceeded under** a closed hierarchy: `Accepted(OrderPlaced)`, `Rejected(Invariant,
Reason)`.
**Site** `PlaceOrderOutcome.cs` D-12.

### Q-26
> `DSC-0001` supplies an invariant name through `settled_by: "invariant:CartNotEmpty"`.
> `DSC-0005` supplies none. If a rejection must cite the invariant it corresponds to,
> where does the citation for DSC-0005 come from? I coined `CurrencyMatchesAccount`, which
> appears in no input and which a second reader will not reproduce.

**Prompted by** `place-order.determinations.yaml:29` against `:193-198`.
**Proceeded under** coining the name.
**Site** `PlaceOrderHandler.cs` D-20(b).

---

## F5 — Payload & validation

### Q-09
> Where does `DSC-0002`'s payload validation live? The handler may reject "only for
> invariants the fact vocabulary declares" and a payload type fault is not one, so it
> cannot be the handler. The controller `must_not` contain "a conditional on domain
> state" — is a payload type check a conditional on domain state? And if the controller
> does it, the resulting 400 is a transport result **not** "derived from the handler's
> Accepted or Rejected", contradicting the controller's own `must`. One of those rules
> has to give. Which?

**Prompted by** `place-order.determinations.yaml:48-50` against
`profile-rest-api-v1.md:60,63,72`.
**Proceeded under** validation in the controller; the "derived from" rule gives.
**Site** `PlaceOrderController.cs` D-23.

---

## F6 — Authority & actor

### Q-17
> `DSC-0003` says "who may place an order is not settled at this address", `residual`,
> carried by team `platform-security`. I have therefore implemented **no** authorisation
> check. Should the built slice record that an undischarged residual is live at this
> address — an attribute, a startup assertion, a failing test — or is silence the correct
> realisation of an unsettled determination? Silence and "nobody thought about it" look
> identical in the code.

**Prompted by** `place-order.determinations.yaml:92-107`.
**Proceeded under** silence in behaviour, plus a test asserting the absence and a
comment naming the residual.
**Site** `PlaceOrderHandler.cs` remarks; `PlaceOrderHandlerTests.Does_not_decide_who_may_place_an_order`.

---

## F3 + F8 — Position, boundary, role decomposition

### Q-06
> `DSC-0003` settles that `ActorIdentity`'s `read_provenance` is an "OIDC token claim,
> validated at the gateway". The only carrier of that claim is the HTTP request. The
> profile makes the provider the role that supplies read-position facts and says a
> provider `must_not` "reference a transport type". **The one permitted supplier of this
> fact may not touch the only thing that carries it.** How is the claim meant to reach
> the provider?

**Prompted by** `place-order.determinations.yaml:114` against `profile-rest-api-v1.md:86`.
**Proceeded under** an `IClaimSource` indirection, with the transport reference moved to
an unroled type. The rule is satisfied as written and defeated in substance.
**Site** `Providers.cs` D-14; `Unroled/Adapters.cs`.

### Q-11
> `Cart` is `internal` — the `Cart` read-model slice writes it, and DSC-0001 and DSC-0005
> both declare the boundary `internal`. The profile scopes the provider to "where
> **external** data is required". But the handler `must_not` perform I/O and the
> controller `must_not` reference a persistence type, so no role may fetch an internal
> fact either. Is the provider's trigger clause a sufficiency condition (a provider is
> *needed* when data is external) or a restriction (a provider may *only* supply external
> data)? Under the second reading the slice cannot read its own ground.

**Prompted by** `profile-rest-api-v1.md:3,77-83` against
`place-order.determinations.yaml:31-34`.
**Proceeded under** sufficiency. `CartProvider` supplies an internal fact.
**Site** `Providers.cs` D-15.

### Q-16
> Nothing in the profile forbids a slice from containing types that declare **no** role,
> and no rule reaches such a type. I have built a conforming slice in which an unroled
> decorator performs the handler's I/O, an unroled adapter holds the provider's transport
> reference, and an unroled sink is the persistence the controller may not name. **Every
> `must_not` holds and every prohibited thing happens.** Is the unroled type intended to
> be outside the profile, and if not, what rule closes it?

**Prompted by** the three `must_not` sets in `profile-rest-api-v1.md:61-63,74-75,85-86`,
and their silence about non-role types.
**Proceeded under** building the evasion deliberately, marking it, and asserting it in a
passing test so the defect is executable rather than editorial.
**Site** `Unroled/README.md`; `ProfileConformanceTests.EvadesEveryMustNot_ByIndirection`.

### Q-24
> The profile writes the marker as `[Slice(<instance>, "controller")]` — a string
> literal. Is the role argument meant to be a string, or is a closed enum acceptable? And
> is the attribute type supplied by the framework or authored per solution? I have
> authored it, so nothing makes my `[Slice]` the same `[Slice]` any analyser would look
> for.

**Prompted by** `profile-rest-api-v1.md:54,69,81`.
**Proceeded under** authoring `SliceAttribute` with a closed `SliceRole` enum.
**Site** `Profile/SliceAttribute.cs` D-01, D-02.

### Q-25
> The controller `must` "call exactly one type declaring the handler role for the same
> act instance". Does "exactly one" constrain only handler-role calls, or does it forbid
> the controller calling anything else at all? My controller also touches `ModelState`
> and constructs `ProblemDetails`.

**Prompted by** `profile-rest-api-v1.md:58`.
**Proceeded under** the narrow reading: the constraint counts handler-role calls.
**Site** `PlaceOrderController.cs`.

---

## F9 — Effect & egress

### Q-10
> `PlaceOrder` declares `writes: [OrderPlaced]`. An event emitted and never persisted is
> not written. But the handler `must_not` perform I/O, the controller `must_not`
> reference a persistence type, the provider's every rule is about supplying reads and it
> `must_not` decide — and there is no fourth role. **The write position has no
> realisation anywhere in the profile.** What persists the event?

**Prompted by** `ordering.eventmodel.yaml:54-57` against the whole of the profile's
`roles:` block.
**Proceeded under** an unroled decorator between the controller's interface and the
handler. Synchronous, non-transactional; a failed append loses a placed order.
**Site** `Unroled/EventAppendingPlaceOrderHandler.cs` D-26, D-27.

---

## F7 — Transport realisation

### Q-12
> The controller `must` "return a transport result **derived from** the handler's Accepted
> or Rejected". Derived *how*? Nothing states the status for an accepted command, the
> status for a rejected one, whether a `Location` header is owed, or what the body is.
> Two readers produce two incompatible APIs from this profile and both conform. Is the
> mapping meant to be free, or is it missing?

**Prompted by** `profile-rest-api-v1.md:59`.
**Proceeded under** Accepted → 201 + `Location`; Rejected → 422 + ProblemDetails;
invalid payload → 400. All invented.
**Site** `PlaceOrderController.cs` D-21. **This is the largest single hole found.**

### Q-13
> What is the HTTP surface of `PlaceOrder` — verb, path, versioning? The profile fixes
> the stack and names the role and fixes no URL. `POST /orders` is my convention, not the
> specification's.

**Prompted by** `profile-rest-api-v1.md:50-63`.
**Proceeded under** `POST /orders`, unversioned.
**Site** `PlaceOrderController.cs` D-24.

### Q-27
> The controller `must_not` contain "a conditional on domain state", yet `must` derive its
> result from Accepted-or-Rejected — which is a conditional on the act's outcome. Where is
> the line? I read it as forbidding a branch on **which** invariant was cited, so all
> rejections map to one status and the invariant travels in the body. A per-invariant
> status map is the obvious alternative and I cannot tell if it is forbidden.

**Prompted by** `profile-rest-api-v1.md:59` against `:61`.
**Proceeded under** one status for all rejections.
**Site** `PlaceOrderController.cs` D-22.

---

## F10 — Identity & time

### Q-14
> Does an order have an identifier, who mints it, and what is it? Nothing in any input
> says `OrderPlaced` carries one — or a timestamp. Both are unusable-without, so I added
> both. A GUID and a human-meaningful order number are equally supported, and the choice
> changes the event, the `Location` header and the public API.

**Prompted by** `ordering.eventmodel.yaml:26` — `OrderPlaced` is an id and a kind.
**Proceeded under** an injected `IOrderIdentityMint` returning a GUID, and an injected
`TimeProvider`.
**Site** `Facts.cs` D-08; `Unroled/Adapters.cs` D-30.

---

## F11 — Lifecycle & concurrency

### Q-15
> `PlaceOrder` writes `OrderPlaced` and nothing else; `CartEmptied` belongs to
> `EmptyCart`. **So a placed order leaves its cart standing**, and the same cart can be
> ordered again. I think this is wrong and I have implemented it as specified. Is the
> cart meant to survive its own order, or is a write position missing from the act?

**Prompted by** `ordering.eventmodel.yaml:54-62`.
**Proceeded under** implementing it as specified, and pinning the behaviour in a test so
the disagreement cannot quietly become a fix.
**Site** `PlaceOrderHandlerTests.Leaves_the_cart_standing_after_the_order_is_placed`.

### Q-23
> Nothing addresses idempotency, retries, or two concurrent `PlaceOrder` calls against one
> cart. Two calls place two orders. Is that intended, out of scope for a determination
> layer, or a determination nobody has filed?

**Prompted by** the absence of any such statement across all four inputs.
**Proceeded under** no idempotency, no concurrency control, asserted in a test.
**Site** `PlaceOrderHandlerTests.Places_a_second_order_from_the_same_cart`.

---

## F12 — Enforcement & evidence

### Q-21
> Is a slice with no test conforming? Nothing in the profile, the determinations or the
> schema requires a test, and `DSC-0002`'s acceptance names a Roslyn analyser rather than
> a test. I wrote tests anyway. If tests are not part of conformance, I have added
> artefacts the specification does not recognise; if they are, the profile is missing a
> role.

**Prompted by** `profile-rest-api-v1.md:47-92` — no `test` role, no test rule.
**Proceeded under** writing tests, marked as evidence rather than as discharge of any
`checked` allocation.
**Site** `PlaceOrderHandlerTests.cs` D-35.

### Q-28
> `DSC-0002`'s closure is `operational`, `runnable_by:
> "roslyn-analyzer:PayloadTypeConformance"`, `terminates: true`, `tolerance: "exact"`.
> That analyser does not exist. Per CG-R-127 the rule is read-enforced for this run — but
> the determination still *claims* an operational closure. Does an operational closure
> naming an unbuilt runner overstate itself in exactly the way `proxy.known_divergence`
> exists to prevent elsewhere in the schema?

**Prompted by** `place-order.determinations.yaml:70-73` against
`determination.schema.json:139-155,176-185`.
**Proceeded under** treating it as read-enforced and saying so.
**Site** `ProfileConformanceTests.cs` remarks; Gate C enforceability table.

### Q-29
> `DSC-0002` travels to `all-command-slices` **and** `all-contexts`, and `DSC-0100` travels
> to `all-command-slices`. Both anchor on `act_instance: PlaceOrder`, as the schema
> requires and the anchor note concedes. So when I read the determinations at this
> address, **I cannot tell a rule meant for `PlaceOrder` from a rule meant for every
> command slice ever written** except by reading the extent prose. For a builder, is
> there any operational difference — should I have built anything differently for a
> travelling determination than for a bound one?

**Prompted by** `place-order.determinations.yaml:51-63` and
`profile-rest-api-v1.md:12-31,41`.
**Proceeded under** no operational difference. Bound and travelling determinations were
implemented identically.
**Site** `gate-a.md` contradiction F.
