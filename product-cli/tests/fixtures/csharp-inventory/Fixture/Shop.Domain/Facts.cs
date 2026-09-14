// The fact vocabulary of examples/ordering.eventmodel.yaml, realised. Names
// deliberately differ from the fact ids where the PRD says they may
// (CartState realises Cart; OrderSummaryView realises OrderSummary).
using Product.Binding;

namespace Shop.Domain;

// A marker interface (no members) and a data contract (properties only):
// the two shapes the denominator criterion's proxies must exclude.
public interface IDomainEvent { }

public interface IHasTotal
{
    Money Total { get; }
}

[RealisesFact("Cart")]
public sealed record CartState(Guid CartId, IReadOnlyList<CartLine> Lines, Money Total) : IHasTotal;

public sealed record CartLine(string Sku, int Quantity, Money Price);

[RealisesFact("ItemAddedToCart")]
public sealed record ItemAddedToCart(Guid CartId, string Sku, int Quantity, Money Price) : IDomainEvent;

[RealisesFact("CartEmptied")]
public sealed record CartEmptied(Guid CartId) : IDomainEvent;

[RealisesFact("OrderPlaced")]
public sealed record OrderPlaced(Guid OrderId, Guid CartId, Money Total) : IHasTotal, IDomainEvent;

[RealisesFact("OrderConfirmed")]
public sealed record OrderConfirmed(Guid OrderId);

[RealisesFact("OrderSummary")]
public sealed record OrderSummaryView(Guid OrderId, Money Total, string Status);

[RealisesFact("ActorIdentity")]
public sealed record ActorIdentity(string Subject);

// A fact the vocabulary does not name: an orphan declaration.
[RealisesFact("Ghost")]
public sealed record Ghost(string Why);

// Shared value object with no fact declaration — referenced from many acts.
// CG-R-52's known divergence lives here: shared is not unstructured.
public readonly record struct Money(decimal Amount, string Currency)
{
    public static Money Zero(string currency) => new(0m, currency);
    public Money Add(Money other) => other.Currency == Currency
        ? new Money(Amount + other.Amount, Currency)
        : throw new InvalidOperationException("currency mismatch");
}
