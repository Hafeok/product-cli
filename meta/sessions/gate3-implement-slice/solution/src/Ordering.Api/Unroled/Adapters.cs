namespace Ordering.Api.Unroled;

using System.Collections.Concurrent;
using Ordering.Api.Facts;
using Ordering.Api.Slices.PlaceOrder;

/// <summary>
/// Feeds <see cref="IClaimSource"/> from the request.
/// </summary>
/// <remarks>
/// NO [Slice] ATTRIBUTE. DELIBERATE — see Unroled/README.md and Q-06.
///
/// DSC-0003 settles that `ActorIdentity` arrives as an "OIDC token claim, validated at
/// the gateway". This type is where that claim is actually read, and it references
/// `IHttpContextAccessor` — a transport type. The provider that supplies the fact may
/// not reference one, so the reference lives here instead, one hop away and outside
/// every profile rule.
///
/// D-28 — DECIDED. The token is trusted as already validated, per DSC-0003's
/// `read_provenance`. Nothing here checks a signature, an issuer, an audience or an
/// expiry. If the gateway is absent in some deployment, this reads an attacker's claim.
/// That risk is the direct consequence of a settled determination and is recorded, not
/// hedged against; hedging would be authoring a determination.
/// </remarks>
public sealed class HttpContextClaimSource : IClaimSource
{
    private readonly IHttpContextAccessor _accessor;

    public HttpContextClaimSource(IHttpContextAccessor accessor) => _accessor = accessor;

    public string? Find(string claimType) =>
        _accessor.HttpContext?.User?.FindFirst(claimType)?.Value;
}

/// <summary>
/// D-29 — INVENTED. In-memory, because no input names a store and inventing a schema
/// would be authoring further than the slice requires. The `Cart` read-model slice is
/// what would populate this; it is out of scope for this run, which builds one slice.
/// </summary>
public sealed class InMemoryCartStore : ICartStore
{
    private readonly ConcurrentDictionary<string, Cart> _carts = new(StringComparer.Ordinal);

    public Cart? Find(string cartId) => _carts.TryGetValue(cartId, out var cart) ? cart : null;

    public void Put(Cart cart) => _carts[cart.CartId] = cart;
}

/// <summary>D-29 — INVENTED. In-memory, same reason.</summary>
public sealed class InMemoryOrderPlacedSink : IOrderPlacedSink
{
    private readonly List<OrderPlaced> _appended = new();

    public IReadOnlyList<OrderPlaced> Appended
    {
        get { lock (_appended) { return _appended.ToArray(); } }
    }

    public void Append(OrderPlaced placed)
    {
        lock (_appended) { _appended.Add(placed); }
    }
}

/// <summary>
/// D-30 — INVENTED. A GUID as the order identifier. Nothing settles that orders have
/// identifiers, let alone their shape, so a human-meaningful order number is equally
/// supported and would change the event, the Location header and the API. Q-14.
/// </summary>
public sealed class GuidOrderIdentityMint : IOrderIdentityMint
{
    public string Next() => Guid.NewGuid().ToString("N");
}

/// <summary>
/// Maps <see cref="ReadPositionUnavailableException"/> to a transport result.
/// </summary>
/// <remarks>
/// NO [Slice] ATTRIBUTE. DELIBERATE.
///
/// D-31 — INVENTED, AND UNSUPPORTED BY ANY INPUT. The profile admits two handler exits,
/// Accepted and Rejected, and the controller derives the transport result from them. A
/// read position that cannot be supplied is neither. The status codes chosen —
/// 401 when `ActorIdentity` is missing, 404 when `Cart` is — encode a security judgement
/// and a REST convention that this session imported wholesale. Q-09, Q-12.
/// </remarks>
public sealed class ReadPositionUnavailableMiddleware
{
    private readonly RequestDelegate _next;

    public ReadPositionUnavailableMiddleware(RequestDelegate next) => _next = next;

    public async Task InvokeAsync(HttpContext context)
    {
        try
        {
            await _next(context);
        }
        catch (ReadPositionUnavailableException ex)
        {
            context.Response.StatusCode = ex.FactType == nameof(ActorIdentity)
                ? StatusCodes.Status401Unauthorized
                : StatusCodes.Status404NotFound;

            await context.Response.WriteAsJsonAsync(new
            {
                title = "A read position could not be supplied.",
                factType = ex.FactType,
            });
        }
    }
}
