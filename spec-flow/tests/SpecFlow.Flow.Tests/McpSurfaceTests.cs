using System.Text.Json;
using ModelContextProtocol.Client;
using ModelContextProtocol.Protocol;
using SpecFlow.Mcp;

namespace SpecFlow.Flow.Tests;

/// <summary>
/// The MCP surface, driven by a real client over the real protocol.
/// </summary>
public class McpSurfaceTests
{
    private static SpecFlowOptions OptionsFor(SpecRepo repo, string? source = null)
        => new(repo.Root, repo.Executable, source);

    private static async Task<IList<McpClientTool>> ToolsAsync(McpHarness harness)
        => await harness.Client.ListToolsAsync();

    private static JsonElement Parse(CallToolResult result)
    {
        var text = result.Content
            .OfType<TextContentBlock>()
            .Select(b => b.Text)
            .FirstOrDefault() ?? "null";
        return JsonDocument.Parse(text).RootElement.Clone();
    }

    [Fact]
    public async Task The_delegable_verbs_are_served()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        var names = (await ToolsAsync(harness)).Select(t => t.Name).ToList();
        foreach (var expected in new[]
                 {
                     "spec_import", "spec_candidates", "spec_map", "spec_check",
                     "spec_records", "spec_policy_show", "spec_implement",
                 })
        {
            Assert.Contains(expected, names);
        }
    }

    [Fact]
    public async Task No_verb_that_names_a_principal_is_served()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        var names = (await ToolsAsync(harness)).Select(t => t.Name).ToHashSet(StringComparer.Ordinal);
        foreach (var withheld in Server.Withheld)
        {
            Assert.DoesNotContain(withheld, names);
        }
    }

    [Fact]
    public async Task Calling_a_withheld_verb_fails_rather_than_resolving_to_something_else()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        await Assert.ThrowsAnyAsync<Exception>(
            async () => await harness.Client.CallToolAsync("spec_close", new Dictionary<string, object?>()));
    }

    [Fact]
    public async Task The_reads_are_marked_read_only_and_the_writes_are_not()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));
        var tools = (await ToolsAsync(harness)).ToDictionary(t => t.Name, StringComparer.Ordinal);

        Assert.True(tools["spec_check"].ProtocolTool.Annotations?.ReadOnlyHint);
        Assert.True(tools["spec_map"].ProtocolTool.Annotations?.ReadOnlyHint);
        Assert.NotEqual(true, tools["spec_implement"].ProtocolTool.Annotations?.ReadOnlyHint);
    }

    [Fact]
    public async Task Import_scans_and_writes_the_inventory()
    {
        using var repo = new SpecRepo();
        using var source = new ImportFixture();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo, source.Root));

        var result = await harness.Client.CallToolAsync("spec_import", new Dictionary<string, object?>());
        var body = Parse(result);

        Assert.True(body.GetProperty("entry_points").GetInt32() > 0);
        Assert.True(File.Exists(Path.Combine(repo.Root, ".spec/inventory.json")));
    }

    [Fact]
    public async Task Import_tells_the_caller_not_to_author_the_model_from_candidates()
    {
        using var repo = new SpecRepo();
        using var source = new ImportFixture();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo, source.Root));

        var body = Parse(await harness.Client.CallToolAsync("spec_import", new Dictionary<string, object?>()));
        var note = body.GetProperty("note").GetString() ?? "";
        Assert.Contains("not a domain model", note, StringComparison.Ordinal);
    }

    [Fact]
    public async Task Check_is_proxied_to_the_rust_gate()
    {
        using var repo = new SpecRepo();
        using var source = new ImportFixture();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo, source.Root));

        await harness.Client.CallToolAsync("spec_import", new Dictionary<string, object?>());
        var body = Parse(await harness.Client.CallToolAsync("spec_check", new Dictionary<string, object?>()));

        var classes = body.GetProperty("structural").EnumerateArray()
            .Select(f => f.GetProperty("class").GetString())
            .ToList();
        Assert.Contains("S005", classes);
    }

    [Fact]
    public async Task Candidates_come_back_with_their_slots_unfilled()
    {
        using var repo = new SpecRepo();
        using var source = new ImportFixture();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo, source.Root));
        await harness.Client.CallToolAsync("spec_import", new Dictionary<string, object?>());

        var body = Parse(await harness.Client.CallToolAsync(
            "spec_candidates", new Dictionary<string, object?> { ["unreviewed"] = true }));

        var first = body.EnumerateArray().First();
        var slots = first.GetProperty("unfilled_slots").EnumerateArray()
            .Select(s => s.GetString()).ToList();
        Assert.Equal(["name", "settles"], slots);
    }

    [Fact]
    public async Task Implement_opens_a_pending_record_and_hands_the_closure_over()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        var body = Parse(await harness.Client.CallToolAsync("spec_implement",
            new Dictionary<string, object?>
            {
                ["slice"] = "checkout-totals",
                ["actRef"] = "act/settle-a-basket",
            }));

        Assert.Equal("pending-closure", body.GetProperty("status").GetString());
        Assert.StartsWith("spec close ", body.GetProperty("hand_off").GetString() ?? "", StringComparison.Ordinal);
        Assert.Equal(1, repo.Check());
    }

    [Fact]
    public async Task Implement_tells_the_caller_the_record_is_not_closed()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        var body = Parse(await harness.Client.CallToolAsync("spec_implement",
            new Dictionary<string, object?> { ["slice"] = "s", ["actRef"] = "act/a" }));

        var note = body.GetProperty("note").GetString() ?? "";
        Assert.Contains("not yours to do", note, StringComparison.Ordinal);
    }

    [Fact]
    public async Task Policy_show_reports_the_default_as_structural_only()
    {
        using var repo = new SpecRepo();
        await using var harness = await McpHarness.StartAsync(OptionsFor(repo));

        var body = Parse(await harness.Client.CallToolAsync(
            "spec_policy_show", new Dictionary<string, object?>()));
        Assert.True(body.GetProperty("in_force").ValueKind is JsonValueKind.Null);
    }
}

/// <summary>The client half: what a slice-building agent is allowed to hold.</summary>
public class GovernedToolTests
{
    [Fact]
    public async Task A_missing_server_degrades_rather_than_fails()
    {
        using var repo = new SpecRepo();
        var tools = await GovernedTools.ConnectAsync(
            new SpecFlowOptions(repo.Root, repo.Executable),
            serverBinary: "definitely-not-on-path-spec-mcp");

        Assert.Empty(tools);
    }

    [Fact]
    public async Task The_tools_an_agent_receives_carry_no_withheld_verb()
    {
        using var repo = new SpecRepo();
        var tools = await GovernedTools.ConnectAsync(
            new SpecFlowOptions(repo.Root, repo.Executable), SpecMcpBinary());

        Assert.NotEmpty(tools);
        var names = tools.Select(t => t.Name).ToHashSet(StringComparer.Ordinal);
        foreach (var withheld in Server.Withheld)
        {
            Assert.DoesNotContain(withheld, names);
        }
    }

    [Fact]
    public async Task An_agent_can_read_the_gate_through_its_tools()
    {
        using var repo = new SpecRepo();
        var tools = await GovernedTools.ConnectAsync(
            new SpecFlowOptions(repo.Root, repo.Executable), SpecMcpBinary());
        Assert.Contains(tools, t => t.Name is "spec_check");
    }

    /// <summary>
    /// The Rust MCP server, required rather than optional.
    /// </summary>
    /// <remarks>
    /// A test that quietly skips when its subject is missing is a test that
    /// passes in exactly the environment where it matters least. CI builds the
    /// whole workspace, so absence here is a broken setup, not a valid state.
    /// </remarks>
    private static string SpecMcpBinary()
    {
        var supplied = Environment.GetEnvironmentVariable("SPEC_MCP_BIN");
        if (!string.IsNullOrWhiteSpace(supplied))
        {
            return supplied;
        }
        for (var probe = new DirectoryInfo(AppContext.BaseDirectory); probe is not null; probe = probe.Parent)
        {
            var candidate = Path.Combine(probe.FullName, "target", "debug", "spec-mcp");
            if (File.Exists(candidate))
            {
                return candidate;
            }
        }
        throw new InvalidOperationException(
            "could not find `spec-mcp`; build it with `cargo build -p spec-mcp` or set SPEC_MCP_BIN");
    }
}
