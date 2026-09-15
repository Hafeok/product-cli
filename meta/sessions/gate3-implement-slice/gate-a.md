# Gate A — before writing any code

**Written with `place-order.determinations.yaml` unopened.** The act vocabulary
(`ordering.eventmodel.yaml`), the profile (`profile-rest-api-v1.md`) and the
schema (`determination.schema.json`) have been read; the determinations file has
not. See `bootstrap.md` for why that is the ordering and what it costs.

---

## 1. What I expect to have to invent

Written first, before the determinations are opened, so the result cannot be
rationalised afterwards. Twenty-one items, ordered roughly by how confident I am
that the specification will not settle them.

### Near-certain — nothing I have read could carry these

1. **Every field of every fact.** `Cart`, `ActorIdentity` and `OrderPlaced` are
   names with a `kind` and, in one case, a one-line note. The fact vocabulary
   declares a type space, not types. I expect to invent every property name,
   every type, and every nullability decision on all three.
2. **The command message itself.** `PlaceOrder` is an act name. No input names a
   `PlaceOrderCommand`, its shape, or its relation to the HTTP request body.
3. **Order identity.** Something must identify the order `OrderPlaced` announces.
   Whether the caller supplies it, the handler mints it, or a provider allocates
   it — and what it is — is settled nowhere I have read.
4. **The `Accepted` / `Rejected` types.** The profile names them four times and
   defines them nowhere. Whether `Accepted` carries the emitted events, whether
   `Rejected` carries a reason, a code, or a cited invariant, whether they are a
   closed hierarchy or a generic result — all mine.
5. **The `[Slice]` attribute.** Three of the profile's `must` rules are
   *"declares `[Slice(<instance>, "role")]`"*. The attribute type is not
   supplied. I must author it: namespace, argument types, whether `role` is a
   string or an enum, its `AttributeUsage`.
6. **Solution and project layout.** Name, .NET version, project count, folder
   convention, namespaces, one-file-per-type or not. The profile says nothing
   about layout; `stack: "C# / ASP.NET Core"` is its entire statement on it.
7. **Async or synchronous signatures**, and `CancellationToken` propagation.
8. **Dependency injection and the composition root.** Three roles are named;
   nothing says how one reaches another, or who registers them.

### Very likely — the profile gestures at these without settling them

9. **The HTTP surface.** Verb, route template, content type. "Controller" is a
   role name, not a route. I expect no input to state that `PlaceOrder` is
   `POST /orders`, and I expect to have to decide it.
10. **The transport mapping.** The controller `must` *"return a transport result
    derived from the handler's Accepted or Rejected"*. **Derived how** is the
    single largest hole I expect to find: 200 vs 201 vs 202 for Accepted, 400 vs
    409 vs 422 for Rejected, whether a `Location` header is required, what the
    response body is. "Derived" names a dependency, not a function.
11. **Where the emitted event goes.** The handler `must` *"emit only events the
    act declares it writes"* and `must_not` *"perform I/O directly"*. So the
    handler produces `OrderPlaced` and cannot persist it. No fourth role
    persists it either — the profile has three. I expect to have to invent the
    entire egress path, and I expect that to be a structural hole rather than an
    omission. See contradiction B below.
12. **How `Cart` reaches the handler.** `Cart` is internal — the `Cart`
    read-model slice writes it. The provider role is scoped to *"where external
    data is required"*. Whether an internal read also goes through a provider, or
    through some unnamed mechanism, is not settled anywhere I have read.
13. **The invariants `PlaceOrder` rejects for.** The handler `must` *"reject only
    for invariants the fact vocabulary declares"*. `ordering.eventmodel.yaml`
    declares no invariants at all. Either the answer is "none, the handler cannot
    reject", or the determinations carry them. This is the item I most expect the
    determinations to settle, and the item whose absence would be most damaging.
    See contradiction C.
14. **Request validation.** A missing body, a malformed id, a negative quantity.
    Is rejecting those a *"conditional on domain state"*, which the controller is
    forbidden? The boundary between transport validation and domain decision is
    not drawn.
15. **Failure behaviour.** What the slice does when a provider cannot supply
    `ActorIdentity` — reject, throw, 503. Not a domain rejection; not covered.

### Likely — real engineering decisions no input appears to reach

16. **Idempotency and concurrency.** Placing the same cart twice; two concurrent
    `PlaceOrder` calls on one cart. Nothing I have read mentions either.
17. **Whether the cart is emptied.** `CartEmptied` exists and `EmptyCart` writes
    it. `PlaceOrder` writes only `OrderPlaced`, so on a plain reading the cart
    survives its own order. I expect to leave it, and to note that it looks wrong.
18. **Persistence at all.** No store, bus, outbox, `DbContext` or repository
    appears in any input. I expect to invent an abstraction and stub it.
19. **Tests.** Nothing requires any. I expect to have to decide whether a slice
    with no test is conforming.
20. **The content hash.** `DSC-0100` pins `"profile:rest-api-v1@<content-hash>"`
    — a literal placeholder. If anything must cite the pinned profile version, I
    must invent or compute the hash.
21. **Money, quantities and units.** If `OrderPlaced` carries a total, its type
    (`decimal`, minor units, currency) is unsettled, and this is the classic
    place a specification's silence becomes a defect.

### What I expect the determinations *to* settle

For the record, so the prediction is falsifiable in both directions. I expect
`place-order.determinations.yaml` to settle: the boundary declarations demanded
by the vocabulary's own notes (`ActorIdentity` external, `OrderConfirmed`
terminal — the file says C-1/C-2 fail otherwise); at least one acceptance
predicate over `PlaceOrder`; and the positions with their `read_provenance` and
`tick_rate`. I expect it to settle **no field of any fact** and **no part of the
HTTP surface**.

---

## 2. My reading of what the slice is

**The act.** `command PlaceOrder`, from `ordering.eventmodel.yaml:54-57`. One
name in a closed act vocabulary of six slices across one context, `ordering`.

**The facts it reads — the ground.**

| Fact | Kind | Written by | Position, per the schema |
|---|---|---|---|
| `Cart` | entity | the `Cart` read-model slice | read → ground; boundary `internal` |
| `ActorIdentity` | entity | *nothing in scope* | read → ground; boundary must be `external` |

`ActorIdentity`'s own note says it is *"present in the vocabulary but produced by
no slice here. A determination must declare it external, or the resolution
condition fails."* Per the schema, `external` obliges `source`,
`read_provenance` and `tick_rate` (`determination.schema.json:209-213`).

**The facts it writes — the verdict.**

| Fact | Kind | Read by | Position |
|---|---|---|---|
| `OrderPlaced` | event | `ConfirmOrder`, `OrderSummary` | write → verdict; boundary `internal` |

`OrderPlaced` is read twice in scope, so it is not terminal. `PlaceOrder` writes
nothing else — notably not `CartEmptied`.

**The positions.** Under DP-6 (`determination.schema.json:189-191`), read and
write are positions relative to this act, not properties of the facts. `Cart`
occupies a read position here and a write position on the `Cart` read-model
slice; the same object, two addresses. So `PlaceOrder`'s position set is exactly
`{Cart: read, ActorIdentity: read, OrderPlaced: write}` — three, and the
boundary kinds above.

**What the profile requires.** `profile-rest-api-v1`, `act_type: command`, stack
C#/ASP.NET Core, pinned by `DSC-0100` which anchors on `PlaceOrder` and travels
to `all-command-slices`. Three roles:

- **controller**, required. Declares `[Slice("PlaceOrder", "controller")]`; calls
  exactly one handler-role type for the same instance; returns a transport result
  derived from Accepted/Rejected. Must not branch on domain state, must not touch
  a provider, must not name a persistence type.
- **handler**, required. Declares `[Slice("PlaceOrder", "handler")]`; one entry
  point taking the command, returning Accepted or Rejected; emits only declared
  events; rejects only for declared invariants. Must not name a transport type,
  must not do I/O.
- **provider**, optional — *"where external data is required"*. `PlaceOrder`
  reads `ActorIdentity`, which is external by the vocabulary's own note, so **a
  provider is required for this slice**. Declares `[Slice("PlaceOrder",
  "provider")]`; reached only from the handler for the same instance; supplies
  only facts in a read position on the act. Must not decide, must not name a
  transport type.

Three further rules are `read_enforced` and, per CG-R-127, **every rule in this
run is read-enforced because no analyser exists** — so the `enforcement: analyser`
marking records an intent, and the report must state which rules would have been
analyser-enforced had Gate 2 run.

**So the slice, concretely:** an HTTP entry point that takes a place-order
request, a handler that decides against `Cart` and `ActorIdentity` and emits
`OrderPlaced` or rejects, and a provider that supplies `ActorIdentity` from
outside the context — with the handler forbidden from doing the I/O that reading
and writing require.

---

## 3. What I believe is contradictory or unimplementable as written

Five, before the determinations are opened. Each is stated as a finding, not
repaired.

**A. The standing rules require reading something the prohibitions forbid.**
*"`canon-governance` holds the in-force rules; read them at the pinned commit and
comply"* against *"read the four arrived inputs and nothing else about the
scheme."* `canon-governance` is not in the bundle and no commit is pinned. Not
satisfiable. This session complied with the prohibitions; the in-force rules are
unread.

**B. No role in the profile may perform the slice's I/O, and no role may write.**
The handler `must_not` *"perform I/O directly"*. The controller `must_not`
*"reference a persistence type"*. The provider exists only to *supply* facts —
every rule on it is about reading (*"every fact it supplies is declared in a read
position on the act"*), and it `must_not` contain a decision. The profile has no
fourth role. Yet the act writes `OrderPlaced`, and an event that is emitted but
never persisted is not written. **The write position has no realisation in the
profile.** This is the profile's largest hole and I expect it to be the finding
of this run.

**C. The handler may reject only for invariants that do not exist.** *"Rejects
only for invariants the fact vocabulary declares."* `ordering.eventmodel.yaml`
declares ids, kinds and two prose notes — no invariant, no predicate, no
constraint, and no field against which one could be stated. Read strictly, the
handler may never reject, which makes `Rejected` dead and the controller's
Accepted/Rejected derivation half-unreachable. Either the rule points at a
vocabulary richer than the one delivered, or the rule is unsatisfiable-by-vacuity.

**D. The pin does not identify what it pins.** `DSC-0100.allocation.settled_by`
is the literal string `"profile:rest-api-v1@<content-hash>"`. A pinned allocation
whose `settled_by` is an unresolved placeholder settles nothing checkable — and
the profile it pins is `[PROPOSED]`, i.e. by its own header not in force, while
the determination filing it is presented as recorded at `2026-09-14T00:00:00Z`.
A filed pin to a proposed artefact is a status contradiction on its face.

**E. `read_enforced` rule 1 contradicts the exercise.** *"The handler's logic is
the behavioural specification, not a realisation of one stated elsewhere."* This
run's entire premise is that a specification is stated elsewhere —
`place-order.determinations.yaml` — and that the handler is built from it by a
reader who may not ask what was meant. Under this rule a conforming handler is
one whose logic is primary; under the session prompt a conforming handler is one
derived from the determinations. Both cannot hold. I will implement to the
prompt and record every place where doing so violates this rule.

**F, minor.** `DSC-0100` addresses `act_instance: PlaceOrder` and travels to
`all-command-slices`, and the anchor note concedes the choice is arbitrary and
privileged (CG-R-53). The consequence for *this* run is specific and worth
naming: **there is no way to tell a rule meant for `PlaceOrder` from a rule meant
for every command**, because the pattern-level determination and any
instance-level determination for `PlaceOrder` share an address.

---

**Hold.** No code has been written. Proceeding to open
`place-order.determinations.yaml`.
