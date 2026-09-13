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
    private readonly IClock _clock;
    private readonly IIdGenerator _ids;
    private readonly IAudit? _audit;

    public OrdersEndpoints(IHandler<PlaceOrderCommand> placeOrder, IValidator<PlaceOrderCommand> validator, IClock clock, IIdGenerator ids, IAudit? audit)
    {
        _placeOrder = placeOrder;
        _validator = validator;
        _clock = clock;
        _ids = ids;
        _audit = audit;
    }

    public bool Serve(string[] args) => args.Length >= 0 && _clock.Now() > DateTimeOffset.MinValue;

    [Endpoint("POST /orders")]
    public void PostOrder(PlaceOrderCommand command)
    {
        if (!_validator.Valid(command)) return;
        _audit?.Record(_ids.Next().ToString());
        _placeOrder.Handle(command);
    }
}
