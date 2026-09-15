using System.Text.Json;
using Microsoft.Agents.AI;

namespace SpecFlow.Flow;

/// <summary>
/// Builds a slice by asking an <see cref="AIAgent"/>, and drafts what arose.
/// </summary>
/// <remarks>
/// <para>
/// This is the delegable half in one class. Everything it produces is a
/// draft: the determinations come back as proposals a reviewer amends and a
/// principal later files. The agent is never told it is closing anything,
/// because it never is.
/// </para>
/// <para>
/// The agent is injected rather than constructed here. Which model, which
/// endpoint, which instructions are arrangement parameters — the thing under
/// test when a small model is asked to hold the same flow a large one does —
/// and baking one in would turn the experiment into a configuration.
/// </para>
/// </remarks>
public sealed class AgentSliceBuilder(AIAgent agent, string? extraInstructions = null)
{
    private readonly AIAgent _agent = agent;
    private readonly string? _extraInstructions = extraInstructions;

    /// <summary>The build step, shaped for the workflow's executor.</summary>
    public async ValueTask<SliceBuilt> BuildAsync(
        ActRecordOpened opened,
        CancellationToken cancellationToken = default)
    {
        var session = await _agent.CreateSessionAsync(cancellationToken).ConfigureAwait(false);
        var response = await _agent
            .RunAsync(Prompt(opened), session, cancellationToken: cancellationToken)
            .ConfigureAwait(false);

        var text = response.Text ?? string.Empty;
        return new SliceBuilt(opened.RecordId, opened.Slice, ParseDeterminations(text), text);
    }

    private string Prompt(ActRecordOpened opened) => $"""
        Build the slice `{opened.Slice}` against the specification act `{opened.ActRef}`.

        While acting, note any determination you had to make that the specification
        did not already settle — a choice a reader of the spec could not have
        predicted from it.

        You are not closing anything. Your determinations are drafts: a person
        reviews them and files them under their own name. Do not claim to have
        filed, accepted, or closed anything.

        End your reply with a JSON array of determination addresses on its own
        line, for example:
        ["det/basket-rounding-is-half-even"]
        An empty array is a legitimate answer and means nothing arose.
        {_extraInstructions}
        """;

    /// <summary>
    /// Pull the trailing JSON array off the reply.
    /// </summary>
    /// <remarks>
    /// A reply with no parsable array yields an empty draft rather than an
    /// error: an unreadable draft must not be able to stop the record from
    /// reaching its reviewer. The reviewer sees the raw notes either way.
    /// </remarks>
    public static IReadOnlyList<string> ParseDeterminations(string reply)
    {
        foreach (var line in reply.Split('\n').Reverse())
        {
            var trimmed = line.Trim();
            if (!trimmed.StartsWith('[') || !trimmed.EndsWith(']'))
            {
                continue;
            }
            try
            {
                return JsonSerializer.Deserialize<string[]>(trimmed) ?? [];
            }
            catch (JsonException)
            {
                // Not the array we were looking for; keep scanning upward.
            }
        }
        return [];
    }
}
