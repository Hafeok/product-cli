namespace Product.Binding;

/// <summary>
/// Declares that the decorated type realises a fact in the fact vocabulary.
/// A type may realise a fact without sharing its name; the declaration
/// connects them, not the spelling.
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct | AttributeTargets.Interface | AttributeTargets.Enum, AllowMultiple = false)]
public sealed class RealisesFactAttribute : Attribute
{
    public RealisesFactAttribute(string fact)
    {
        Fact = fact;
    }

    public string Fact { get; }
}
