using Microsoft.AspNetCore.Components;
using Microsoft.Extensions.Logging;
using Shop.Api.Orders;

namespace Shop.Api.Admin;

// A component the framework constructs and satisfies by property injection: each
// [Inject] property is a composition edge by the criterion (CG-R-68). One resolves
// through a registration (ICartReader), one is a boundary edge (ILogger<T>).
public sealed class OrdersPanel : ComponentBase
{
    [Inject] public ICartReader Cart { get; set; } = default!;

    [Inject] public ILogger<OrdersPanel> Logger { get; set; } = default!;

    public int Count(Guid cartId) => Cart.Read(cartId).Lines.Count;
}
