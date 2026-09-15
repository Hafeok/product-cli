namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;
using Ordering.Api.Profile;

/// <summary>
/// The contract the controller calls. Kept separate from the handler type so that the
/// egress decorator (Unroled/) can sit between them without the controller referencing
/// it — see D-19, which is a finding about the profile, not a design flourish.
/// </summary>
public interface IPlaceOrderHandler
{
    PlaceOrderOutcome Handle(PlaceOrderCommand command);
}

/// <summary>
/// D-17 — INVENTED. Nothing settles that an order has an identifier or who mints it, and
/// a handler that may not perform I/O may not reach for a database sequence. Injecting
/// the mint keeps the decision function deterministic under test. Q-14.
/// </summary>
public interface IOrderIdentityMint
{
    string Next();
}

/// <summary>
/// The <c>PlaceOrder</c> decision.
/// </summary>
/// <remarks>
/// PROFILE / handler — required: true.
///   must     "declares [Slice(&lt;instance&gt;, \"handler\")]"                      — below.
///   must     "exposes a single entry point taking the command and returning
///             Accepted or Rejected"                                                — Handle.
///   must     "emits only events the act declares it writes"                        — OrderPlaced only.
///   must     "rejects only for invariants the fact vocabulary declares"            — SEE Q-03.
///   must_not "references a transport type"                                         — holds.
///   must_not "performs I/O directly"                                               — holds; providers supply, decorator writes.
///
/// Q-03 IS LIVE IN THIS FILE AND IS THE MOST IMPORTANT CONFLICT IN THE RUN.
/// The profile permits rejection "only for invariants the FACT VOCABULARY declares".
/// `ordering.eventmodel.yaml` declares ids, kinds and two prose notes. It declares no
/// invariant, no predicate, no constraint, and no field one could be stated over. Read
/// strictly, this handler may never reject, `Rejected` is dead, and the controller's
/// Accepted-or-Rejected mapping is half unreachable.
/// Both rejections below come from the DETERMINATION layer, not the fact vocabulary:
/// DSC-0001 (pinned, `settled_by: "invariant:CartNotEmpty"`) and DSC-0005. Implementing
/// them violates the profile rule as written. Not implementing them ignores a pinned
/// determination addressed to this exact act.
/// This session implements them and reports the violation, on the reading that the rule
/// means "invariants declared over the fact vocabulary" and that "fact vocabulary" is
/// loose for the specification as a whole. That reading is not in the text. Q-03.
///
/// D-18 — DECIDED: rejection ORDER. DSC-0001 is checked before DSC-0005. Nothing
/// settles precedence between two rejections that can both apply to one cart, and the
/// caller sees only the first. A cart that is both empty and mis-currencied reports
/// CartNotEmpty. Reversing it is equally supported by the specification.
///
/// SETTLED by DSC-0003 — AUTHORITY IS DELIBERATELY ABSENT BELOW.
/// "Who may place an order is not settled at this address", allocation `residual`,
/// carried_by `human`, principal team `platform-security`. So there is no authorisation
/// check in this handler, and its absence is a reading of the specification rather than
/// an oversight. The residual is live and someone carries it. Q-17.
///
/// NOT SETTLED ANYWHERE, AND NOT DONE (see decisions.md):
///   * the cart is not emptied. `PlaceOrder` writes `OrderPlaced` and nothing else; the
///     act vocabulary gives `CartEmptied` to `EmptyCart`. So a placed order leaves its
///     cart standing. This session believes that is wrong and implements it as
///     specified. Q-15.
///   * no idempotency and no concurrency control. Two calls place two orders. Q-23.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Handler)]
public sealed class PlaceOrderHandler : IPlaceOrderHandler
{
    private readonly CartProvider _carts;
    private readonly ActorIdentityProvider _actors;
    private readonly IOrderIdentityMint _mint;
    private readonly TimeProvider _clock;

    public PlaceOrderHandler(
        CartProvider carts,
        ActorIdentityProvider actors,
        IOrderIdentityMint mint,
        TimeProvider clock)
    {
        _carts = carts;
        _actors = actors;
        _mint = mint;
        _clock = clock;
    }

    /// <summary>The single entry point the profile requires.</summary>
    public PlaceOrderOutcome Handle(PlaceOrderCommand command)
    {
        // Read positions, both through provider-role types (Providers.cs).
        var actor = _actors.Supply();
        var cart = _carts.Supply(command.CartId);

        // D-19 — DECIDED. A read position that cannot be supplied is not a domain
        // rejection: no determination declares it and the profile forbids rejecting for
        // anything undeclared. So it throws and an unroled middleware maps it. Throwing
        // from a handler the profile describes as returning "Accepted or Rejected" is a
        // third exit the profile does not admit. Q-09, Q-12.
        if (actor is null)
        {
            throw new ReadPositionUnavailableException(nameof(ActorIdentity));
        }

        if (cart is null)
        {
            throw new ReadPositionUnavailableException(nameof(Cart));
        }

        // DSC-0001 — pinned, settled_by "invariant:CartNotEmpty".
        // "An order may not be placed against an empty cart."
        if (cart.IsEmpty)
        {
            return new PlaceOrderOutcome.Rejected(
                "CartNotEmpty",
                "An order may not be placed against an empty cart.");
        }

        // DSC-0005 — act-time, residual, carried by human principal "emil".
        // "An order placed against a cart whose currency differs from the customer's
        //  account currency is rejected rather than converted."
        //
        // D-20 — DECIDED, TWICE OVER.
        //  (a) That a `residual` determination is IMPLEMENTED at all. Its allocation
        //      says a human carries it, with no pin and no check. This session reads
        //      allocation as how a determination is discharged, not whether it holds,
        //      so the statement is behaviour and it is realised here — unverified.
        //      The opposite reading (residual means nothing is built) is available and
        //      would leave this branch out entirely. Q-04.
        //  (b) The invariant NAME. DSC-0001 supplies one via `settled_by`; DSC-0005
        //      supplies none, and `Rejected` must cite something. "CurrencyMatchesAccount"
        //      is this session's coinage and appears in no input.
        //
        // The comparison itself reads a fact DSC-0005 declares no position for; see
        // Facts.cs D-07 and Q-05. String equality, ordinal: case and ISO-4217 validity
        // are unsettled.
        if (!string.Equals(cart.Currency, actor.AccountCurrency, StringComparison.Ordinal))
        {
            return new PlaceOrderOutcome.Rejected(
                "CurrencyMatchesAccount",
                "An order placed against a cart whose currency differs from the "
                + "customer's account currency is rejected rather than converted.");
        }

        // The write position: OrderPlaced, and nothing else.
        var placed = new OrderPlaced(
            OrderId: _mint.Next(),
            CartId: cart.CartId,
            ActorId: actor.ActorId,
            Currency: cart.Currency,
            Lines: cart.Lines,
            TotalMinorUnits: cart.TotalMinorUnits,
            OccurredAt: _clock.GetUtcNow());

        return new PlaceOrderOutcome.Accepted(placed);
    }
}

/// <summary>D-19 — INVENTED. See PlaceOrderHandler.Handle.</summary>
public sealed class ReadPositionUnavailableException : Exception
{
    public ReadPositionUnavailableException(string factType)
        : base($"The read position '{factType}' could not be supplied.")
        => FactType = factType;

    public string FactType { get; }
}
