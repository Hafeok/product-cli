namespace Ordering.Api.Profile;

/// <summary>
/// The role marker every profile <c>rest-api-v1</c> role rule is stated in terms of:
/// <c>declares [Slice(&lt;instance&gt;, "controller")]</c>, and the same for
/// <c>"handler"</c> and <c>"provider"</c>.
/// </summary>
/// <remarks>
/// D-01 — INVENTED. The profile states three <c>must</c> rules in terms of this
/// attribute and never supplies it: not its namespace, not its argument types, not
/// whether the role argument is a string or a closed enum, not its AttributeUsage.
/// Everything below this line is this session's construction and nothing in the four
/// arrived inputs constrains it.
///
/// D-02 — The role argument is typed as a closed enum rather than the bare string the
/// profile's rule text shows. The profile writes the rule as
/// <c>[Slice(&lt;instance&gt;, "controller")]</c> — a string literal. A closed enum is
/// strictly more checkable and the profile's three role names are a closed set. This
/// session judged that the profile is stating the rule, not the signature, and that an
/// enum satisfies it. If the string form was meant literally, this is a divergence.
/// Q-24.
///
/// D-03 — The act instance is a string, unvalidated. Nothing ties it to the act
/// vocabulary in <c>ordering.eventmodel.yaml</c> at compile time. Checking
/// "the instance names an act in the vocabulary" is exactly the kind of rule the absent
/// analyser would carry; see the Gate C enforceability table.
/// </remarks>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct, AllowMultiple = false, Inherited = false)]
public sealed class SliceAttribute : Attribute
{
    public SliceAttribute(string actInstance, SliceRole role)
    {
        ActInstance = actInstance;
        Role = role;
    }

    /// <summary>A name in the act vocabulary, e.g. <c>PlaceOrder</c>.</summary>
    public string ActInstance { get; }

    /// <summary>Which of the profile's three roles this type plays.</summary>
    public SliceRole Role { get; }
}

/// <summary>The profile's role set for <c>act_type: command</c>. Closed: it names three.</summary>
public enum SliceRole
{
    Controller,
    Handler,
    Provider,
}
