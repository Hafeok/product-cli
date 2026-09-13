// A hand-rolled container and a local endpoint attribute, so the fixture
// stays free of framework packages while exercising the two shapes a root
// convention may name: an implemented interface resolved through DI, and a
// declared attribute on a handler method.
namespace Shop.Api.Infrastructure;

[AttributeUsage(AttributeTargets.Method)]
public sealed class EndpointAttribute : Attribute
{
    public EndpointAttribute(string route) { Route = route; }
    public string Route { get; }
}

public interface IHandler<in TCommand>
{
    void Handle(TCommand command);
}

public sealed class Container
{
    private readonly Dictionary<Type, Func<Container, object>> _factories = new();

    public Container Register<TService, TImpl>(Func<Container, TImpl> factory) where TImpl : TService
    {
        _factories[typeof(TService)] = c => factory(c)!;
        return this;
    }

    public T Resolve<T>() => (T)_factories[typeof(T)](this);
}
