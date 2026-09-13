using Shop.Api.Infrastructure;
using Shop.Api.Orders;
using Shop.Api.Persistence;

namespace Shop.Api;

public static class Program
{
    public static int Main(string[] args)
    {
        var container = new Container()
            .Register<IOrderRepository, OrderRepository>(_ => new OrderRepository())
            .Register<ICartReader, CartService>(_ => new CartService())
            .Register<IHandler<PlaceOrderCommand>, PlaceOrderHandler>(
                c => new PlaceOrderHandler(c.Resolve<IOrderRepository>(), c.Resolve<ICartReader>()));
        var endpoints = new OrdersEndpoints(container);
        return endpoints.Serve(args) ? 0 : 1;
    }
}
