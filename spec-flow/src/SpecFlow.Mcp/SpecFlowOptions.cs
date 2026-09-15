namespace SpecFlow.Mcp;

/// <summary>Where the server acts, and what it drives.</summary>
/// <param name="Root">The repo holding <c>.spec/</c>.</param>
/// <param name="SpecBinary">
/// The Rust <c>spec</c> binary. Every read goes through it rather than through
/// a second reader in this process: one implementation of the store's meaning,
/// so an MCP client and a CI run cannot be told different things.
/// </param>
/// <param name="Source">
/// The codebase <c>import</c> scans. Defaults to <see cref="Root"/>.
/// </param>
public sealed record SpecFlowOptions(string Root, string SpecBinary, string? Source = null)
{
    /// <summary>What <c>import</c> scans.</summary>
    public string ScanRoot => Source ?? Root;

    /// <summary>Read from the environment, for a server launched by a client.</summary>
    public static SpecFlowOptions FromEnvironment() => new(
        Environment.GetEnvironmentVariable("SPEC_ROOT") ?? ".",
        Environment.GetEnvironmentVariable("SPEC_BIN") ?? "spec",
        Environment.GetEnvironmentVariable("SPEC_SOURCE"));
}
