using Microsoft.Agents.AI.Workflows;

namespace SpecFlow.Flow;

/// <summary>
/// The <c>implement</c> verb as a deterministic graph.
/// </summary>
/// <remarks>
/// <para>
/// The shape is the argument. The record opens first, the build happens
/// second, the reviewer sees a draft third, and the terminal node renders a
/// command rather than running one. There is no edge from this graph to a
/// closure, so "a slice cannot be closed unattended" is a property of the
/// topology instead of a rule someone has to keep in mind.
/// </para>
/// <para>
/// Why a workflow and not an agent loop: the traversal is authored here, so
/// the arrangement — what is delivered, in what order, with what gate — is
/// ours rather than a planner's. An autonomous runtime would leave the
/// ordering to its own context management, which is the one thing this flow
/// cannot cede.
/// </para>
/// </remarks>
public static class ImplementWorkflow
{
    /// <summary>The typed port a reviewer answers on.</summary>
    public static RequestPort<ClosureDraft, DraftReview> ReviewPort { get; } =
        RequestPort.Create<ClosureDraft, DraftReview>(ExecutorIds.ReviewPort);

    /// <summary>
    /// Build the graph.
    /// </summary>
    /// <param name="cli">The gateway to the Rust store. Cannot close.</param>
    /// <param name="build">
    /// How a slice gets built. Injected so the arrangement stays a parameter.
    /// </param>
    public static Workflow Build(
        SpecCli cli,
        Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> build)
    {
        var openRecord = new OpenRecordExecutor(cli);
        var buildSlice = new BuildSliceExecutor(build);
        var draftClosure = new DraftClosureExecutor();
        var handOff = new HandOffExecutor();

        return new WorkflowBuilder(openRecord)
            .WithName("spec-implement")
            .WithDescription("Build a slice, open its act-time record, hand the closure to a principal")
            .AddEdge(openRecord, buildSlice)
            .AddEdge(buildSlice, draftClosure)
            .AddEdge(draftClosure, ReviewPort)
            .AddEdge(ReviewPort, handOff)
            .WithOutputFrom(handOff)
            .Build();
    }
}
