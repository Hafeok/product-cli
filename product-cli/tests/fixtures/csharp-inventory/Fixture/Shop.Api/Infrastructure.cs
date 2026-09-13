// A local endpoint attribute, so the fixture exercises the attribute-root
// convention without a web framework, plus a handler interface resolved
// through the real Microsoft.Extensions.DependencyInjection container.
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

// Registered with an open-generic typeof pair — the shape the resolver must
// match by generic definition rather than by closed type.
public interface IValidator<T>
{
    bool Valid(T value);
}

public sealed class AlwaysValid<T> : IValidator<T>
{
    public bool Valid(T value) => true;
}

// Referenced by a reached type and registered by nothing: the no-registration case.
public interface IClock
{
    DateTimeOffset Now();
}

// Registered through a factory lambda the reader can see construct it.
public interface IIdGenerator
{
    Guid Next();
}

public sealed class GuidGenerator : IIdGenerator
{
    public Guid Next() => Guid.NewGuid();
}

// Registered only inside a conditional.
public interface IAudit
{
    void Record(string what);
}

public sealed class ConsoleAudit : IAudit
{
    public void Record(string what) => Console.WriteLine(what);
}
