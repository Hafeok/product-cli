namespace Product.Binding;

/// <summary>
/// Declares that the decorated type realises one role of one act instance
/// under a profile. Act, role and profile are strings resolved against the
/// act vocabulary and the profile store at check time — never an enum
/// (CG-R-54: a compiled role vocabulary would be R-C violated).
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct | AttributeTargets.Interface | AttributeTargets.Method, AllowMultiple = true)]
public sealed class SliceAttribute : Attribute
{
    public SliceAttribute(string act, string role)
    {
        Act = act;
        Role = role;
    }

    public string Act { get; }
    public string Role { get; }
    public string? Profile { get; set; }
}
