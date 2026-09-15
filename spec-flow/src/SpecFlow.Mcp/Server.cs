using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace SpecFlow.Mcp;

/// <summary>Hosts the specification flow's MCP surface over stdio.</summary>
public static class Server
{
    /// <summary>
    /// Verbs this surface must never carry, each because it names a principal.
    /// </summary>
    /// <remarks>
    /// Listed rather than merely absent, so a test can fail the moment the list
    /// and the registered tools disagree. The list is belt; <see
    /// cref="Flow.SpecCli.ForbiddenVerbs"/> is braces; the assembly containing
    /// no code that writes a closure is the trousers.
    /// </remarks>
    public static readonly IReadOnlySet<string> Withheld =
        new HashSet<string>(StringComparer.Ordinal)
        {
            "spec_accept",
            "spec_reject",
            "spec_close",
            "spec_policy_set",
        };

    /// <summary>Serve MCP over stdio until the client disconnects.</summary>
    public static async Task RunStdioAsync(
        SpecFlowOptions options,
        CancellationToken cancellationToken = default)
    {
        var builder = Host.CreateEmptyApplicationBuilder(settings: null);

        // stdout is the protocol channel. Anything else written there corrupts
        // the stream, so logging goes to stderr and nowhere else.
        builder.Logging.AddConsole(console => console.LogToStandardErrorThreshold = LogLevel.Trace);

        builder.Services.AddSingleton(options);
        builder.Services
            .AddMcpServer()
            .WithStdioServerTransport()
            .WithTools<SpecFlowTools>();

        await builder.Build().RunAsync(cancellationToken).ConfigureAwait(false);
    }
}
