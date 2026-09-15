namespace SpecFlow.Cli;

/// <summary>What the host was asked to build.</summary>
internal sealed record Options(
    string Root,
    string Slice,
    string ActRef,
    string By,
    string SpecBinary,
    string? Instructions)
{
    public const string Usage = """
        specflow --slice <id> --act <act-ref> [--root <path>] [--by <identity>]
                 [--spec <path to the spec binary>] [--instructions <text>]

        Builds a slice and opens its act-time record. Always exits 3: the
        closure is a principal's act, and this process is not one.
        """;

    public static Options? Parse(string[] args)
    {
        var values = new Dictionary<string, string>(StringComparer.Ordinal);
        for (var i = 0; i + 1 < args.Length; i += 2)
        {
            if (!args[i].StartsWith("--", StringComparison.Ordinal))
            {
                return null;
            }
            values[args[i][2..]] = args[i + 1];
        }

        if (!values.TryGetValue("slice", out var slice) || !values.TryGetValue("act", out var actRef))
        {
            return null;
        }

        return new Options(
            values.GetValueOrDefault("root", "."),
            slice,
            actRef,
            values.GetValueOrDefault("by", "agent@example.invalid"),
            values.GetValueOrDefault("spec", "spec"),
            values.GetValueOrDefault("instructions"));
    }
}
