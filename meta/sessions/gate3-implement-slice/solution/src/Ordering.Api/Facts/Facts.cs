namespace Ordering.Api.Facts;

// ---------------------------------------------------------------------------
// D-04 — INVENTED, ENTIRELY.
//
// `ordering.eventmodel.yaml` is a FACT VOCABULARY: it declares a type space, not
// types. Every fact in it is an `id`, a `kind`, and in two cases a prose `note`.
// No field of `Cart`, `ActorIdentity` or `OrderPlaced` is declared anywhere in the
// four arrived inputs, and no determination addressed to `PlaceOrder` names one.
//
// Every property below is this session's invention. Their names, their types, their
// nullability and their units are unconstrained by the specification, and a second
// reader building the same slice from the same inputs would not produce these.
//
// This matters beyond aesthetics: DSC-0002 requires that "for every field of the
// command payload, a domain type is declared and the supplied value satisfies its
// constraints". There are no declared domain types. The check therefore validates
// this session's inventions against themselves. Q-07.
// ---------------------------------------------------------------------------

/// <summary>Fact <c>Cart</c>, kind <c>entity</c>, "the customer's assembled selection".</summary>
/// <remarks>
/// Read position on <c>PlaceOrder</c>, boundary <c>internal</c> — DSC-0001 and DSC-0005
/// both declare it so. Written by the <c>Cart</c> read-model slice.
/// </remarks>
public sealed record Cart(
    string CartId,
    string Currency,
    IReadOnlyList<CartLine> Lines)
{
    /// <summary>
    /// The predicate DSC-0001 is settled by: <c>settled_by: "invariant:CartNotEmpty"</c>.
    /// </summary>
    /// <remarks>
    /// D-05 — "Empty" is INVENTED. DSC-0001 says "an order may not be placed against an
    /// empty cart" and pins the invariant by name. It does not say what empty means,
    /// and it could not, because <c>Cart</c> has no declared structure. This session
    /// reads empty as "no lines". A cart holding only zero-quantity lines is NOT empty
    /// under this reading, which is a defensible alternative this session rejected
    /// without authority. Q-03.
    /// </remarks>
    public bool IsEmpty => Lines.Count == 0;

    /// <summary>
    /// D-06 — INVENTED. Money is unspecified in every input. Minor units in a long is
    /// this session's choice; decimal, a Money value object, or a per-line currency are
    /// all equally consistent with the specification, which says nothing.
    /// </summary>
    public long TotalMinorUnits => Lines.Sum(line => line.UnitAmountMinorUnits * line.Quantity);
}

/// <summary>D-04 — INVENTED. No input declares that a cart has lines, or what a line is.</summary>
public sealed record CartLine(string Sku, int Quantity, long UnitAmountMinorUnits);

/// <summary>
/// Fact <c>ActorIdentity</c>, kind <c>entity</c>. The vocabulary's own note: "present in
/// the vocabulary but produced by no slice here. A determination must declare it
/// external, or the resolution condition fails." DSC-0003 declares it external.
/// </summary>
/// <remarks>
/// SETTLED by DSC-0003's position record, and this is the one place the specification
/// reaches further than this session expected:
///   boundary.kind            = external
///   boundary.source          = "identity provider, outside the modelled scope"
///   boundary.read_provenance = "OIDC token claim, validated at the gateway"
///   boundary.tick_rate       = fast
/// So: a claim on an already-validated token, not a lookup, and it may change between
/// reads. The provider below does not re-validate; the gateway did.
///
/// D-07 — <c>AccountCurrency</c> is INVENTED AND CONTESTED. DSC-0005 rejects an order
/// "against a cart whose currency differs from the customer's account currency". No
/// fact in the vocabulary carries an account currency; there is no Account fact at all;
/// and DSC-0005's own <c>positions</c> list declares only <c>Cart: read</c> — it
/// declares no position for the second thing its statement compares against. Hanging it
/// off ActorIdentity is this session's choice among several bad ones, and it silently
/// widens a read position the determination did not declare. Q-05.
/// </remarks>
public sealed record ActorIdentity(
    string ActorId,
    string AccountCurrency);

/// <summary>
/// Fact <c>OrderPlaced</c>, kind <c>event</c>. The sole write position on
/// <c>PlaceOrder</c>; boundary <c>internal</c> per DSC-0001, and indeed
/// <c>ConfirmOrder</c> and <c>OrderSummary</c> both read it.
/// </summary>
/// <remarks>
/// D-04 — every field INVENTED. D-08 — <c>OrderId</c> and <c>OccurredAt</c> in
/// particular: nothing in any input says an order has an identifier, who mints it, or
/// that the event is timestamped. Both were added because the event is unusable
/// without them, which is a judgement, not a reading. Q-14.
/// </remarks>
public sealed record OrderPlaced(
    string OrderId,
    string CartId,
    string ActorId,
    string Currency,
    IReadOnlyList<CartLine> Lines,
    long TotalMinorUnits,
    DateTimeOffset OccurredAt);
