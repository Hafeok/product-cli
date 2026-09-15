namespace Ordering.Api.Tests;

using Ordering.Api.Facts;
using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;
using Xunit;

/// <summary>
/// One test per determination addressed to <c>command PlaceOrder</c>, plus the profile
/// rules that can be checked from outside an analyser.
/// </summary>
/// <remarks>
/// D-35 — DECIDED. Nothing in the four inputs requires a test, forbids one, or says
/// whether a slice with no test conforms. These exist because a determination nobody
/// can see fail is indistinguishable from one nobody implemented. Q-21.
///
/// These are NOT the acceptances. DSC-0002's acceptance names
/// <c>runnable_by: "roslyn-analyzer:PayloadTypeConformance"</c>, which does not exist;
/// per CG-R-127 every rule is read-enforced for this run and these tests are evidence
/// about the implementation, not discharge of a checked allocation.
/// </remarks>
public sealed class PlaceOrderHandlerTests
{
    private const string Gbp = "GBP";

    private static PlaceOrderHandler Handler(Cart? cart, ActorIdentity? actor)
    {
        var store = new InMemoryCartStore();
        if (cart is not null)
        {
            store.Put(cart);
        }

        return new PlaceOrderHandler(
            new CartProvider(store),
            new ActorIdentityProvider(new StubClaimSource(actor)),
            new StubMint("order-1"),
            TimeProvider.System);
    }

    private static Cart CartWith(params CartLine[] lines) =>
        new("cart-1", Gbp, lines);

    /// <summary>DSC-0001 — "An order may not be placed against an empty cart."</summary>
    [Fact]
    public void Rejects_an_empty_cart_citing_CartNotEmpty()
    {
        var outcome = Handler(CartWith(), new ActorIdentity("actor-1", Gbp))
            .Handle(new PlaceOrderCommand { CartId = "cart-1" });

        var rejected = Assert.IsType<PlaceOrderOutcome.Rejected>(outcome);
        Assert.Equal("CartNotEmpty", rejected.Invariant);
    }

    /// <summary>
    /// DSC-0005 — "…rejected rather than converted." The test asserts the rejection and
    /// that no converted amount appears; "rather than converted" is otherwise
    /// unobservable from the outcome. D-20.
    /// </summary>
    [Fact]
    public void Rejects_a_currency_mismatch_rather_than_converting()
    {
        var outcome = Handler(CartWith(new CartLine("sku-1", 2, 500)), new ActorIdentity("actor-1", "EUR"))
            .Handle(new PlaceOrderCommand { CartId = "cart-1" });

        var rejected = Assert.IsType<PlaceOrderOutcome.Rejected>(outcome);
        Assert.Equal("CurrencyMatchesAccount", rejected.Invariant);
    }

    /// <summary>D-18 — the decided precedence between two applicable rejections.</summary>
    [Fact]
    public void Reports_CartNotEmpty_first_when_both_rejections_apply()
    {
        var outcome = Handler(new Cart("cart-1", "EUR", Array.Empty<CartLine>()), new ActorIdentity("actor-1", Gbp))
            .Handle(new PlaceOrderCommand { CartId = "cart-1" });

        Assert.Equal("CartNotEmpty", Assert.IsType<PlaceOrderOutcome.Rejected>(outcome).Invariant);
    }

    /// <summary>The write position: <c>OrderPlaced</c> and nothing else.</summary>
    [Fact]
    public void Emits_OrderPlaced_when_the_cart_is_placeable()
    {
        var outcome = Handler(CartWith(new CartLine("sku-1", 2, 500)), new ActorIdentity("actor-1", Gbp))
            .Handle(new PlaceOrderCommand { CartId = "cart-1" });

        var accepted = Assert.IsType<PlaceOrderOutcome.Accepted>(outcome);
        Assert.Equal("order-1", accepted.Event.OrderId);
        Assert.Equal("actor-1", accepted.Event.ActorId);
        Assert.Equal(1000, accepted.Event.TotalMinorUnits);
    }

    /// <summary>
    /// DSC-0003 — "Who may place an order is not settled at this address", residual,
    /// carried by team platform-security. The slice must therefore NOT decide it. This
    /// asserts the absence: an arbitrary actor is not turned away on authority grounds.
    /// </summary>
    [Fact]
    public void Does_not_decide_who_may_place_an_order()
    {
        var outcome = Handler(CartWith(new CartLine("sku-1", 1, 100)), new ActorIdentity("some-stranger", Gbp))
            .Handle(new PlaceOrderCommand { CartId = "cart-1" });

        Assert.IsType<PlaceOrderOutcome.Accepted>(outcome);
    }

    /// <summary>
    /// Q-15 — the cart survives its own order, because the act vocabulary gives
    /// <c>CartEmptied</c> to <c>EmptyCart</c> and <c>PlaceOrder</c> writes only
    /// <c>OrderPlaced</c>. This session believes that is wrong and pins the specified
    /// behaviour here so the belief cannot quietly become a fix.
    /// </summary>
    [Fact]
    public void Leaves_the_cart_standing_after_the_order_is_placed()
    {
        var store = new InMemoryCartStore();
        store.Put(CartWith(new CartLine("sku-1", 1, 100)));

        var handler = new PlaceOrderHandler(
            new CartProvider(store),
            new ActorIdentityProvider(new StubClaimSource(new ActorIdentity("actor-1", Gbp))),
            new StubMint("order-1"),
            TimeProvider.System);

        handler.Handle(new PlaceOrderCommand { CartId = "cart-1" });

        Assert.NotNull(store.Find("cart-1"));
        Assert.False(store.Find("cart-1")!.IsEmpty);
    }

    /// <summary>Q-23 — no idempotency is specified and none is implemented.</summary>
    [Fact]
    public void Places_a_second_order_from_the_same_cart()
    {
        var handler = Handler(CartWith(new CartLine("sku-1", 1, 100)), new ActorIdentity("actor-1", Gbp));
        var command = new PlaceOrderCommand { CartId = "cart-1" };

        Assert.IsType<PlaceOrderOutcome.Accepted>(handler.Handle(command));
        Assert.IsType<PlaceOrderOutcome.Accepted>(handler.Handle(command));
    }

    /// <summary>D-19 — an unsuppliable read position is a third exit the profile does not admit.</summary>
    [Fact]
    public void Throws_when_a_read_position_cannot_be_supplied()
    {
        var handler = Handler(CartWith(new CartLine("sku-1", 1, 100)), actor: null);

        var ex = Assert.Throws<ReadPositionUnavailableException>(
            () => handler.Handle(new PlaceOrderCommand { CartId = "cart-1" }));

        Assert.Equal(nameof(ActorIdentity), ex.FactType);
    }

    private sealed class StubClaimSource : IClaimSource
    {
        private readonly ActorIdentity? _actor;

        public StubClaimSource(ActorIdentity? actor) => _actor = actor;

        public string? Find(string claimType) => claimType switch
        {
            "sub" => _actor?.ActorId,
            "account_currency" => _actor?.AccountCurrency,
            _ => null,
        };
    }

    private sealed class StubMint : IOrderIdentityMint
    {
        private readonly string _id;

        public StubMint(string id) => _id = id;

        public string Next() => _id;
    }
}
