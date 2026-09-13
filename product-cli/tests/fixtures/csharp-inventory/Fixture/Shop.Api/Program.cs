using Microsoft.Extensions.DependencyInjection;
using Shop.Api.Infrastructure;
using Shop.Api.Orders;
using Shop.Api.Persistence;

namespace Shop.Api;

public static class Program
{
    public static int Main(string[] args)
    {
        var services = new ServiceCollection();
        services.AddScoped<IOrderRepository, OrderRepository>();
        services.AddScoped<ICartReader, CartService>();
        services.AddScoped<IHandler<PlaceOrderCommand>, PlaceOrderHandler>();
        services.AddScoped(typeof(IValidator<>), typeof(AlwaysValid<>));
        services.AddSingleton<IIdGenerator>(_ => new GuidGenerator());
        if (args.Length > 0)
        {
            services.AddSingleton<IAudit, ConsoleAudit>();
        }
        services.AddScoped<OrdersEndpoints>();
        using var provider = services.BuildServiceProvider();
        var endpoints = provider.GetRequiredService<OrdersEndpoints>();
        return endpoints.Serve(args) ? 0 : 1;
    }
}
