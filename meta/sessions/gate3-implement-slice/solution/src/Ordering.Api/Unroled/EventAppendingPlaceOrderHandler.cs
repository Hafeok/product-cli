namespace Ordering.Api.Unroled;

using Ordering.Api.Facts;
using Ordering.Api.Slices.PlaceOrder;

/// <summary>
/// D-26 — INVENTED. No input names an event store, a stream, a bus or an outbox.
/// </summary>
public interface IOrderPlacedSink
{
    void Append(OrderPlaced placed);
}

/// <summary>
/// Writes the act's verdict.
/// </summary>
/// <remarks>
/// NO [Slice] ATTRIBUTE. DELIBERATE. THIS IS THE FINDING — see Unroled/README.md.
///
/// The act `PlaceOrder` declares `writes: [OrderPlaced]`. An event that is emitted and
/// never persisted is not written, so the write position obliges someone to persist it.
/// In `profile-rest-api-v1` that someone does not exist:
///   * the handler `must_not` "performs I/O directly";
///   * the controller `must_not` "references a persistence type";
///   * the provider supplies read-position facts — every rule on it is about reading,
///     it `must_not` contain a decision, and nothing in the profile gives it a write;
///   * there is no fourth role.
/// So this type is unroled, and the profile therefore says nothing about it at all.
/// Q-10, Q-16.
///
/// D-27 — DECIDED. The sink is called synchronously inside the request, between the
/// handler returning and the controller mapping. Transactionality, ordering, retry and
/// what happens when the append fails after the decision succeeded are all unsettled and
/// all unaddressed. A failed append here loses a placed order.
/// </remarks>
public sealed class EventAppendingPlaceOrderHandler : IPlaceOrderHandler
{
    private readonly PlaceOrderHandler _inner;
    private readonly IOrderPlacedSink _sink;

    public EventAppendingPlaceOrderHandler(PlaceOrderHandler inner, IOrderPlacedSink sink)
    {
        _inner = inner;
        _sink = sink;
    }

    public PlaceOrderOutcome Handle(PlaceOrderCommand command)
    {
        var outcome = _inner.Handle(command);

        if (outcome is PlaceOrderOutcome.Accepted accepted)
        {
            _sink.Append(accepted.Event);
        }

        return outcome;
    }
}
