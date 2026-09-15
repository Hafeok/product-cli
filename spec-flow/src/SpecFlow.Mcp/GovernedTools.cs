using Microsoft.Extensions.AI;
using ModelContextProtocol.Client;

namespace SpecFlow.Mcp;

/// <summary>
/// The only tools a slice-building agent gets: the flow's own read surface,
/// consumed over MCP.
/// </summary>
/// <remarks>
/// <para>
/// An agent holding only these can act only through governed surfaces, so
/// escape through un-governed tooling is structurally excluded rather than
/// discouraged. What is not excluded is the planner's own context handling,
/// and that is an honest limit rather than a gap to paper over.
/// </para>
/// <para>
/// The server is the Rust <c>spec-mcp</c> binary, which is the same gate CI
/// runs. An agent reading the store through a second implementation could be
/// told something CI disagrees with, and would then be right to be confused.
/// </para>
/// </remarks>
public static class GovernedTools
{
    /// <summary>
    /// Connect to <c>spec-mcp</c> and return its tools, or nothing when the
    /// binary is absent.
    /// </summary>
    /// <remarks>
    /// Absence degrades rather than fails: a repo without the Rust gate built
    /// can still open records, and the record — not the agent's reading — is
    /// what the write-back leg depends on.
    /// </remarks>
    public static async Task<IReadOnlyList<AITool>> ConnectAsync(
        SpecFlowOptions options,
        string serverBinary = "spec-mcp",
        CancellationToken cancellationToken = default)
    {
        try
        {
            var client = await McpClient.CreateAsync(
                new StdioClientTransport(new StdioClientTransportOptions
                {
                    Name = "spec-mcp",
                    Command = serverBinary,
                    Arguments = [options.Root],
                }),
                cancellationToken: cancellationToken).ConfigureAwait(false);

            var tools = await client.ListToolsAsync(cancellationToken: cancellationToken)
                .ConfigureAwait(false);
            return [.. tools.Where(IsPermitted).Cast<AITool>()];
        }
        catch (Exception e) when (e is IOException or InvalidOperationException
                                       or System.ComponentModel.Win32Exception)
        {
            return [];
        }
    }

    /// <summary>
    /// Whether a tool the server offered may reach the agent.
    /// </summary>
    /// <remarks>
    /// The server already withholds every verb that names a principal. This
    /// filter assumes it might not: a surface that trusts what it is handed is
    /// a surface that inherits the other side's next mistake.
    /// </remarks>
    public static bool IsPermitted(McpClientTool tool) => !Server.Withheld.Contains(tool.Name);
}
