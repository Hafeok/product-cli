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

// An interface that declares nothing and inherits its members: read as a marker
// before v4 (O-13); a service once members are counted over the chain (P-1).
public interface IReadStore<T>
{
    T Load(Guid id);
}

public interface IAuditStore : IReadStore<string>
{
}

public sealed class MemoryAuditStore : IAuditStore
{
    public string Load(Guid id) => id.ToString();
}

// Registered by a call chained off AddHealthChecks() — a builder registration
// (R-2): the health check service constructs it from the container.
public sealed class PingCheck : Microsoft.Extensions.Diagnostics.HealthChecks.IHealthCheck
{
    private readonly IClockFactory _clocks;

    public PingCheck(IClockFactory clocks) { _clocks = clocks; }

    public Task<Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckResult> CheckHealthAsync(
        Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckContext context, CancellationToken cancellationToken = default)
        => Task.FromResult(_clocks.Create().Now() > DateTimeOffset.MinValue
            ? Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckResult.Healthy()
            : Microsoft.Extensions.Diagnostics.HealthChecks.HealthCheckResult.Unhealthy());
}

// Registered, and resolved by no edge: container-constructed all the same
// (O-17 — the registration list decides, not whether something asks for it).
// Also: a collection parameter (every registration of IIdGenerator, P-6) and a
// Func<> provider (the criterion's partial row, P-5 — provider ids only).
public sealed class AuditSink
{
    private readonly IAudit _audit;
    private readonly IAuditStore _store;
    private readonly IEnumerable<IIdGenerator> _generators;
    private readonly Func<IAudit> _lateAudit;

    public AuditSink(IAudit audit, IAuditStore store, IEnumerable<IIdGenerator> generators, Func<IAudit> lateAudit)
    {
        _audit = audit; _store = store; _generators = generators; _lateAudit = lateAudit;
    }

    public void Flush(Guid id) => _lateAudit().Record(_store.Load(id) + _generators.Count() + _audit.GetHashCode());
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
