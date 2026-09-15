# `Unroled/` — the types the profile does not reach

Every type in this folder is part of the `PlaceOrder` slice and **declares no
`[Slice]` role**, so not one rule in `profile-rest-api-v1` applies to it.

This folder is a **finding**, not a layer.

The profile constrains three roles and has no rule about types outside them, and
no rule forbidding a slice from having such types. The consequence is direct:

* The handler `must_not` *"perform I/O directly"*. It does not.
  `EventAppendingPlaceOrderHandler` here does the I/O, on the handler's behalf,
  in the same call stack, and is unconstrained.
* The provider `must_not` *"reference a transport type"*. It does not.
  `HttpContextClaimSource` here reads `HttpContext` and hands the claim to the
  provider through an interface, and is unconstrained.
* The controller `must_not` *"reference a persistence type"*. It does not. It
  calls `IPlaceOrderHandler`, which DI resolves to the appending decorator.

**Every `must_not` in the profile is satisfied as written and defeated in
substance, by indirection alone, with no cleverness.** This is the run's first
concrete evidence for PRD §11.4: an analyser enforcing these rules against role
declarations would pass this solution, and the three `must_not` rules marked
`enforcement: analyser` are checkable *literally* but not *in effect*. See Q-16
and the Gate C enforceability table.

The `does the write position have a home` question is separate and worse: the
profile has three roles, none of which may persist the event the act declares it
writes, and no fourth. The write position is unrealisable inside the profile.
See Q-10.
