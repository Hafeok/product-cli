using System.IO.Pipelines;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using ModelContextProtocol.Client;
using ModelContextProtocol.Protocol;
using ModelContextProtocol.Server;
using SpecFlow.Mcp;

namespace SpecFlow.Flow.Tests;

/// <summary>
/// A real client and a real server, paired over in-memory pipes.
/// </summary>
/// <remarks>
/// The protocol round-trip is what is being tested, not a mock of it. A
/// reflection test over the tool attributes proves the surface is declared
/// correctly; only a client actually calling <c>tools/list</c> proves it is
/// *served* correctly, and the two have drifted in every SDK ever shipped.
/// </remarks>
public sealed class McpHarness : IAsyncDisposable
{
    private readonly McpServer _server;
    private readonly Task _serverLoop;
    private readonly CancellationTokenSource _cancellation = new();

    public McpClient Client { get; private set; } = null!;

    private McpHarness(McpServer server)
    {
        _server = server;
        _serverLoop = server.RunAsync(_cancellation.Token);
    }

    /// <summary>Start a server over <paramref name="options"/> and connect to it.</summary>
    public static async Task<McpHarness> StartAsync(SpecFlowOptions options)
    {
        var clientToServer = new Pipe();
        var serverToClient = new Pipe();

        // Registered through the same DI path production uses, so the test
        // exercises the real registration rather than a parallel list of tools
        // that could quietly diverge from it.
        var services = new ServiceCollection();
        services.AddSingleton(options);
        services.AddMcpServer(o => o.ServerInfo = new Implementation { Name = "specflow", Version = "test" })
            .WithTools<SpecFlowTools>();
        var provider = services.BuildServiceProvider();
        var serverOptions = provider.GetRequiredService<IOptions<McpServerOptions>>().Value;

        var transport = new StreamServerTransport(
            clientToServer.Reader.AsStream(),
            serverToClient.Writer.AsStream(),
            "specflow");

        var server = McpServer.Create(transport, serverOptions, serviceProvider: provider);
        var harness = new McpHarness(server);

        var clientTransport = new StreamClientTransport(
            clientToServer.Writer.AsStream(),
            serverToClient.Reader.AsStream());
        harness.Client = await McpClient.CreateAsync(clientTransport);
        return harness;
    }

    public async ValueTask DisposeAsync()
    {
        await _cancellation.CancelAsync();
        await Client.DisposeAsync();
        await _server.DisposeAsync();
        try
        {
            await _serverLoop;
        }
        catch (OperationCanceledException)
        {
            // Shutting a server down is not a failure.
        }
        _cancellation.Dispose();
    }
}
