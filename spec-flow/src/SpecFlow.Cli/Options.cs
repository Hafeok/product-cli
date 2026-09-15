namespace SpecFlow.Cli;

/// <summary>A parsed command line: a verb plus its flags.</summary>
internal sealed record Options(string Verb, IReadOnlyDictionary<string, string> Values)
{
    public const string Usage = """
        specflow <verb> [flags]

        Verbs — the delegable half of the specification flow. Ratification and
        closure are not here; they are `spec accept` and `spec close`, and they
        name a principal.

          import     --root <path> [--source <path>]
                     Re-scan a C# codebase into .spec/inventory.json.

          implement  --slice <id> --act <act-ref> [--root <path>] [--by <identity>]
                     Build a slice and open its act-time record. Always exits 3:
                     the closure is a principal's act, and this process is not one.

          mcp        [--root <path>] [--spec <path>] [--source <path>]
                     Serve the delegable verbs over MCP on stdio. accept, reject,
                     close and policy set are withheld — each names a principal.

        Shared flags:
          --root <path>          repo holding .spec/ (default: .)
          --spec <path>          the spec binary (default: `spec` on PATH)
        """;

    public string Root => Values.GetValueOrDefault("root", ".");
    public string SpecBinary => Values.GetValueOrDefault("spec", "spec");
    public string? Get(string key) => Values.GetValueOrDefault(key);

    /// <summary>
    /// Parse a verb followed by <c>--key value</c> pairs.
    /// </summary>
    /// <returns><c>null</c> when the shape is wrong, so the caller can print usage.</returns>
    public static Options? Parse(string[] args)
    {
        if (args.Length is 0 || args[0].StartsWith("--", StringComparison.Ordinal))
        {
            return null;
        }

        var values = new Dictionary<string, string>(StringComparer.Ordinal);
        for (var i = 1; i < args.Length; i += 2)
        {
            if (!args[i].StartsWith("--", StringComparison.Ordinal) || i + 1 >= args.Length)
            {
                return null;
            }
            values[args[i][2..]] = args[i + 1];
        }
        return new Options(args[0], values);
    }
}
