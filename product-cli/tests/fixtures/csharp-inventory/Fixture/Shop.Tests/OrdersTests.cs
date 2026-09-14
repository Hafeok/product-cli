using Microsoft.Extensions.DependencyInjection;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Shop.Api.Infrastructure;

namespace Shop.Tests;

// A test project: its registrations and implementors are test composition,
// not production composition (P-3). IClock is registered here and nowhere
// else; the primary convention must still see it as registered by nothing.
[TestClass]
public sealed class OrdersTests
{
    [TestMethod]
    public void Clock_is_wired_for_tests()
    {
        var services = new ServiceCollection();
        services.AddSingleton<IClock, FakeClock>();
        services.AddSingleton<IAudit, FakeAudit>();
        using var provider = services.BuildServiceProvider();
        Assert.IsNotNull(provider.GetRequiredService<IClock>());
    }
}

public sealed class FakeClock : IClock
{
    public DateTimeOffset Now() => DateTimeOffset.UnixEpoch;
}

public sealed class FakeAudit : IAudit
{
    public void Record(string what) { }
}
