namespace Ordering.Api.Slices.PlaceOrder;

using Ordering.Api.Facts;
using Ordering.Api.Profile;

/// <summary>
/// Everything a provider needs that is not itself a profile role.
/// </summary>
/// <remarks>
/// D-13 — INVENTED. No input names a store, a stream, a repository or a claims source.
/// These two abstractions exist so the providers have something to be reached through;
/// their implementations are unroled (see Unroled/).
/// </remarks>
public interface ICartStore
{
    Cart? Find(string cartId);
}

/// <summary>
/// The already-validated claim set for the act in progress.
/// </summary>
/// <remarks>
/// D-14 — INVENTED, AND THIS IS THE PROFILE'S SHARPEST CONFLICT, NOT A CONVENIENCE.
///
/// DSC-0003 settles that <c>ActorIdentity</c>'s <c>read_provenance</c> is an
/// "OIDC token claim, validated at the gateway". The only place a request's token claim
/// exists is the HTTP request. The profile says the provider is the role that supplies
/// read-position facts, and in the same breath says a provider <c>must_not</c>
/// "reference a transport type". The one permitted supplier of this fact may not touch
/// the only thing that carries it.
///
/// This interface is the indirection that makes the conflict disappear from the
/// provider's source text without making it disappear from the program: the provider
/// references <c>IClaimSource</c>, and an unroled type in the composition root feeds it
/// from <c>HttpContext</c>. The rule is satisfied as written and defeated in substance.
/// That is reported as a finding, not offered as a solution. Q-06, Q-16.
/// </remarks>
public interface IClaimSource
{
    string? Find(string claimType);
}

/// <summary>
/// Supplies the <c>Cart</c> read position.
/// </summary>
/// <remarks>
/// PROFILE / provider — required: false, "where external data is required".
///
/// D-15 — DECIDED. <c>Cart</c> is <c>internal</c>, not external: the <c>Cart</c>
/// read-model slice writes it, and DSC-0001 and DSC-0005 both declare its boundary
/// <c>internal</c>. So the profile's trigger for a provider ("where external data is
/// required") does not fire for this fact. But the handler <c>must_not</c> "perform I/O
/// directly", and the controller <c>must_not</c> "reference a persistence type", so no
/// other role may fetch it either. This session read the provider's trigger clause as a
/// sufficiency condition rather than a restriction on what a provider may supply — the
/// provider's own <c>must</c> rules are satisfied exactly, including "every fact it
/// supplies is declared in a read position on the act". A reading under which internal
/// facts may not go through a provider leaves the slice with no way to read its own
/// ground. Q-11.
///
/// PROFILE / provider must_not "contains a decision": returning null when the cart is
/// absent is a lookup outcome, not a decision. This session believes that holds; it is
/// the kind of line the absent analyser would have to draw.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Provider)]
public sealed class CartProvider
{
    private readonly ICartStore _store;

    public CartProvider(ICartStore store) => _store = store;

    public Cart? Supply(string cartId) => _store.Find(cartId);
}

/// <summary>
/// Supplies the <c>ActorIdentity</c> read position.
/// </summary>
/// <remarks>
/// PROFILE / provider. Required here: <c>ActorIdentity</c> is external by the
/// vocabulary's own note and by DSC-0003, so the profile's "where external data is
/// required, a provider" fires.
///
/// SETTLED by DSC-0003: the fact arrives as a claim on a token already validated at the
/// gateway, so nothing here validates a token. <c>tick_rate: fast</c> is why this reads
/// per act rather than caching.
///
/// D-16 — INVENTED. The claim TYPES are not settled. DSC-0003 says "OIDC token claim"
/// and names no claim. <c>sub</c> for the actor is the OIDC-conventional choice, which
/// is a convention this session imported and the specification did not supply.
/// <c>account_currency</c> is an outright invention with no OIDC basis at all — see
/// Facts.cs D-07 and Q-05. A different reader gets different claim names here and the
/// slice silently reads nothing.
/// </remarks>
[Slice("PlaceOrder", SliceRole.Provider)]
public sealed class ActorIdentityProvider
{
    private readonly IClaimSource _claims;

    public ActorIdentityProvider(IClaimSource claims) => _claims = claims;

    public ActorIdentity? Supply()
    {
        var actorId = _claims.Find("sub");
        var accountCurrency = _claims.Find("account_currency");

        if (string.IsNullOrWhiteSpace(actorId) || string.IsNullOrWhiteSpace(accountCurrency))
        {
            return null;
        }

        return new ActorIdentity(actorId, accountCurrency);
    }
}
