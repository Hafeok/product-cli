using System.ClientModel;
using Microsoft.Agents.AI;
using OpenAI;
using OpenAI.Chat;
using SpecFlow.Flow;

namespace SpecFlow.Cli;

/// <summary>The <c>implement</c> verb: build a slice, hand the closure over.</summary>
internal static class ImplementCommand
{
    public static async Task<int> RunAsync(Options options)
    {
        var slice = options.Get("slice");
        var actRef = options.Get("act");
        if (slice is null || actRef is null)
        {
            Console.Error.WriteLine("implement needs --slice and --act");
            return ExitCodes.CouldNotRun;
        }

        var cli = new SpecCli(options.SpecBinary, options.Root);
        var driver = new ImplementDriver(cli, BuildStrategy(options), ReviewAtTheTerminal);
        var outcome = await driver
            .RunAsync(new ImplementRequest(slice, actRef, options.Get("by") ?? "agent@example.invalid"))
            .ConfigureAwait(false);

        Report(outcome);
        return ExitCodes.PendingClosure;
    }

    /// <summary>
    /// How the slice gets built. With no model endpoint configured the run
    /// still happens and still opens a record — the drafting is what is
    /// missing, not the write-back leg.
    /// </summary>
    private static Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> BuildStrategy(Options options)
    {
        var endpoint = Environment.GetEnvironmentVariable("SPECFLOW_MODEL_ENDPOINT");
        if (string.IsNullOrWhiteSpace(endpoint))
        {
            return (opened, _) => new ValueTask<SliceBuilt>(
                new SliceBuilt(opened.RecordId, opened.Slice, [], "no model endpoint configured; nothing drafted"));
        }

        var model = Environment.GetEnvironmentVariable("SPECFLOW_MODEL") ?? "gpt-4o-mini";
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
        return new AgentSliceBuilder(agent, options.Get("instructions")).BuildAsync;
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
        IReadOnlyList<string> determinations = typed switch
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
