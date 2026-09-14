using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Product.Binding;
using Shop.Api.Infrastructure;
using Shop.Api.Persistence;
using Shop.Domain;

namespace Shop.Api.Orders;

public sealed record PlaceOrderCommand(Guid CartId, string Subject, Money Budget);

// Declared: act PlaceOrder, role handler, under rest-api-v1. Reads Cart and
// ActorIdentity, writes OrderPlaced — exactly the slice's positions.
[Slice("PlaceOrder", "handler", Profile = "rest-api-v1")]
public sealed class PlaceOrderHandler : IHandler<PlaceOrderCommand>
{
    private readonly IOrderRepository _orders;
    private readonly ICartReader _carts;

    public PlaceOrderHandler(IOrderRepository orders, ICartReader carts)
    {
        _orders = orders;
        _carts = carts;
    }

    public void Handle(PlaceOrderCommand command)
    {
        CartState cart = _carts.Read(command.CartId);
        var actor = new ActorIdentity(command.Subject);
        if (cart.Lines.Count == 0) throw new InvalidOperationException("empty cart");
        var placed = new OrderPlaced(Guid.NewGuid(), cart.CartId, cart.Total);
        _orders.Save(placed, actor);
    }
}

// Undeclared, and its referenced facts (Cart, ItemAddedToCart, CartEmptied)
// are covered by exactly one act: the Cart read-model. Declarable.
public interface ICartReader
{
    CartState Read(Guid cartId);
}

public sealed class CartService : ICartReader
{
    private readonly Dictionary<Guid, CartState> _carts = new();

    public CartState Read(Guid cartId) =>
        _carts.TryGetValue(cartId, out var cart) ? cart : new CartState(cartId, Array.Empty<CartLine>(), Money.Zero("DKK"));

    public void Apply(ItemAddedToCart added)
    {
        var cart = Read(added.CartId);
        var lines = cart.Lines.Append(new CartLine(added.Sku, added.Quantity, added.Price)).ToList();
        _carts[added.CartId] = cart with { Lines = lines, Total = cart.Total.Add(added.Price) };
    }

    public void Apply(CartEmptied emptied) => _carts.Remove(emptied.CartId);

    // A type test on the marker: chooses nothing, resolves nothing.
    public bool IsEmptying(IDomainEvent e) => e is CartEmptied;

    // A data contract as a method parameter: a shape, not a dependency.
    public static Money TotalOf(IHasTotal carrier) => carrier.Total;
}

// Undeclared, and no single act covers Cart + OrderPlaced + OrderConfirmed:
// PlaceOrder reads Cart and writes OrderPlaced, ConfirmOrder reads OrderPlaced
// and writes OrderConfirmed. The act boundary runs through this type.
public sealed class CheckoutService
{
    private readonly ICartReader _carts;
    private readonly IOrderRepository _orders;

    public CheckoutService(ICartReader carts, IOrderRepository orders)
    {
        _carts = carts;
        _orders = orders;
    }

    public OrderConfirmed PlaceAndConfirm(Guid cartId, string subject)
    {
        CartState cart = _carts.Read(cartId);
        var placed = new OrderPlaced(Guid.NewGuid(), cart.CartId, cart.Total);
        _orders.Save(placed, new ActorIdentity(subject));
        return new OrderConfirmed(placed.OrderId);
    }
}

// Declared against an act the vocabulary does not name.
[Slice("Nonexistent", "handler", Profile = "rest-api-v1")]
public sealed class BogusHandler
{
    public void Handle(PlaceOrderCommand command) { }
}

public sealed class OrdersEndpoints
{
    private readonly IHandler<PlaceOrderCommand> _placeOrder;
    private readonly IValidator<PlaceOrderCommand> _validator;
    private readonly IClockFactory _clocks;
    private readonly IServiceProvider _provider;
    private readonly IComparer<Money> _byAmount;
    private readonly IAudit? _audit;
    private readonly IReadOnlyList<string> _routes;
    private readonly ILogger<OrdersEndpoints> _log;

    // IReadOnlyList<string> is a data contract (properties only) and IDomainEvent a
    // marker: both arrive as constructor parameters here so the criterion's
    // excluded roles are exercised at a composition edge. ILogger<T> is an external
    // abstraction nothing in the solution implements or registers: a boundary edge (CG-R-68).
    public OrdersEndpoints(IHandler<PlaceOrderCommand> placeOrder, IValidator<PlaceOrderCommand> validator, IClockFactory clocks, IServiceProvider provider, IComparer<Money> byAmount, IAudit? audit, IReadOnlyList<string> routes, IDomainEvent? last, ILogger<OrdersEndpoints> log)
    {
        _routes = routes;
        _log = log;
        _ = last;
        _placeOrder = placeOrder;
        _validator = validator;
        _clocks = clocks;
        _provider = provider;
        _byAmount = byAmount;
        _audit = audit;
    }

    public bool Serve(string[] args) => _log is not null && args.Length >= 0 && _routes.Count >= 0 && _clocks.Create().Now() > DateTimeOffset.MinValue
        && _byAmount.Compare(Money.Zero("DKK"), Money.Zero("DKK")) == 0;

    [Endpoint("POST /orders")]
    public void PostOrder(PlaceOrderCommand command)
    {
        if (!_validator.Valid(command)) return;
        var ids = _provider.GetRequiredService<IIdGenerator>();
        _audit?.Record(ids.Next().ToString());
        _placeOrder.Handle(command);
    }
}
