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

// A factory abstraction: composition chooses the factory, the factory
// chooses later — the criterion's partial row.
public interface IClockFactory
{
    IClock Create();
}

public sealed class SystemClock : IClock
{
    public DateTimeOffset Now() => DateTimeOffset.UtcNow;
}

public sealed class SystemClockFactory : IClockFactory
{
    public IClock Create() => new SystemClock();
}

// An external abstraction (System.Collections.Generic.IComparer<T>) with an
// in-solution registration and an in-solution consumer.
public sealed class MoneyComparer : IComparer<Shop.Domain.Money>
{
    public int Compare(Shop.Domain.Money x, Shop.Domain.Money y) => x.Amount.CompareTo(y.Amount);
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
