using Shop.Domain;

namespace Shop.Api.Persistence;

public interface IOrderRepository
{
    void Save(OrderPlaced placed, ActorIdentity by);
}

public sealed class OrderRepository : IOrderRepository
{
    private readonly List<(OrderPlaced, ActorIdentity)> _rows = new();

    public void Save(OrderPlaced placed, ActorIdentity by) => _rows.Add((placed, by));

    internal int Count => _rows.Count;
}

// Reached by nothing: the isolated case.
public sealed class Dead
{
    public string Nothing() => "unreached";
}

internal static class Helpers
{
    public static string Trim(string s) => s.Trim();
}
