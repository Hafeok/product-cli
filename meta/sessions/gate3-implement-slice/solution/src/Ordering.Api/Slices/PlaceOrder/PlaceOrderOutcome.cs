namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;

/// <summary>
/// The handler's return type. The profile names <c>Accepted</c> and <c>Rejected</c> four
/// times — in the handler's entry-point rule and in the controller's transport rule —
/// and defines neither.
/// </summary>
/// <remarks>
/// D-12 — INVENTED. That <c>Accepted</c> carries the emitted events, that
/// <c>Rejected</c> carries a cited invariant and a reason, and that the pair is a closed
/// hierarchy rather than a generic Result, are all this session's constructions.
///
/// <c>Accepted</c> carrying the events is forced, not chosen: the handler <c>must</c>
/// "emit only events the act declares it writes" and <c>must_not</c> "perform I/O
/// directly". A handler that may emit but may not write has nowhere to put the event
/// except its own return value. See Unroled/EventAppendingPlaceOrderHandler.cs — the
/// egress has no home in the profile at all. Q-10.
///
/// <c>Rejected</c> carrying <c>Invariant</c> is this session's reading of the profile's
/// read-enforced rule 2: "a rejection reason corresponds to the invariant it cites, not
/// merely to a declared one". A rejection that cites an invariant needs a field to cite
/// it in. Q-22.
/// </remarks>
public abstract record PlaceOrderOutcome
{
    private PlaceOrderOutcome() { }

    /// <summary>The act's verdict: the events it wrote.</summary>
    public sealed record Accepted(OrderPlaced Event) : PlaceOrderOutcome;

    /// <summary>
    /// The act declined. <paramref name="Invariant"/> is the invariant identifier the
    /// rejection cites; <paramref name="Reason"/> is the corresponding statement.
    /// </summary>
    public sealed record Rejected(string Invariant, string Reason) : PlaceOrderOutcome;
}
