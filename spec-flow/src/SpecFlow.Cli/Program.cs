using System.ClientModel;
using Microsoft.Agents.AI;
using OpenAI;
using OpenAI.Chat;
using SpecFlow.Flow;

namespace SpecFlow.Cli;

/// <summary>
/// The agent host: the delegable half of the specification flow.
/// </summary>
/// <remarks>
/// It builds slices and hands closures over. It cannot close one — not by
/// policy, but because nothing in the assemblies it links offers the verb.
/// </remarks>
internal static class Program
{
    private const int PendingClosure = 3;
    private const int CouldNotRun = 2;

    private static async Task<int> Main(string[] args)
    {
        var options = Options.Parse(args);
        if (options is null)
        {
            Console.Error.WriteLine(Options.Usage);
            return CouldNotRun;
        }

        try
        {
            var outcome = await RunAsync(options).ConfigureAwait(false);
            Report(outcome);
            return PendingClosure;
        }
        catch (Exception e) when (e is InvalidOperationException or IOException)
        {
            Console.Error.WriteLine(e.Message);
            return CouldNotRun;
        }
    }

    private static async Task<ImplementOutcome> RunAsync(Options options)
    {
        var cli = new SpecCli(options.SpecBinary, options.Root);
        var build = BuildStrategy(options);
        var driver = new ImplementDriver(cli, build, ReviewAtTheTerminal);
        return await driver
            .RunAsync(new ImplementRequest(options.Slice, options.ActRef, options.By))
            .ConfigureAwait(false);
    }

    /// <summary>
    /// How the slice gets built. With no model endpoint configured the run
    /// still happens and still opens a record — the drafting is what is
    /// missing, not the write-back leg.
    /// </summary>
    private static Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> BuildStrategy(Options options)
    {
        var endpoint = Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT");
        var model = Environment.GetEnvironmentVariable("SPECFLOW_MODEL") ?? "gpt-4o-mini";
        if (string.IsNullOrWhiteSpace(endpoint))
        {
            return (opened, _) => new ValueTask<SliceBuilt>(
                new SliceBuilt(opened.RecordId, opened.Slice, [], "no model endpoint configured; nothing drafted"));
        }

        var key = Environment.GetEnvironmentVariable("SPECFLOW_MODEL_KEY") ?? "not-needed";
        var client = new OpenAIClient(
            new ApiKeyCredential(key),
            new OpenAIClientOptions { Endpoint = new Uri(endpoint) });
        AIAgent agent = client.GetChatClient(model).AsAIAgent(new ChatClientAgentOptions
        {
            // Stable logical-role id: a checkpoint can only resume into a graph
            // whose executor identities match, and a random id per run makes
            // every checkpoint its own unresumable lineage.
            Id = "slice-builder",
            Name = "SliceBuilder",
        });
        var builder = new AgentSliceBuilder(agent, options.Instructions);
        return builder.BuildAsync;
    }

    /// <summary>
    /// The human-in-the-loop port, answered at a terminal.
    /// </summary>
    /// <remarks>
    /// The reviewer amends a draft here. They do not close the record here,
    /// and the prompt says so: the closure is a separate act, under their own
    /// identity, at the other binary.
    /// </remarks>
    private static ValueTask<DraftReview> ReviewAtTheTerminal(ClosureDraft draft, CancellationToken cancellationToken)
    {
        Console.WriteLine($"\nrecord {draft.RecordId}  slice `{draft.Slice}`");
        Console.WriteLine(draft.Notes);
        Console.WriteLine("\ndrafted determinations:");
        foreach (var determination in draft.DraftDeterminations)
        {
            Console.WriteLine($"  {determination}");
        }
        if (draft.DraftDeterminations.Count is 0)
        {
            Console.WriteLine("  (none)");
        }

        Console.Write("\namend (comma-separated addresses, blank to keep, `-` for none): ");
        var typed = Console.ReadLine();
        var determinations = typed switch
        {
            null or "" => draft.DraftDeterminations,
            "-" => [],
            _ => typed.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries),
        };
        return new ValueTask<DraftReview>(new DraftReview(draft.RecordId, determinations));
    }

    private static void Report(ImplementOutcome outcome)
    {
        Console.WriteLine($"\nrecord {outcome.RecordId} is open. Closure is pending and is not this process's to do.");
        Console.WriteLine("Run this yourself, under your own identity:\n");
        Console.WriteLine($"  {outcome.HandOffCommand}\n");
    }
}
