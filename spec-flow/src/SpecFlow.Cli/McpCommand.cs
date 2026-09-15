using SpecFlow.Mcp;

namespace SpecFlow.Cli;

/// <summary>The <c>mcp</c> verb: serve the delegable surface over stdio.</summary>
internal static class McpCommand
{
    public static async Task<int> RunAsync(Options options)
    {
        await Server
            .RunStdioAsync(new SpecFlowOptions(options.Root, options.SpecBinary, options.Get("source")))
            .ConfigureAwait(false);
        return ExitCodes.Conformant;
    }
}
