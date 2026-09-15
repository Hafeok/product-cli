using Microsoft.Agents.AI.Workflows;

namespace SpecFlow.Flow;

/// <summary>
/// Stable executor ids.
/// </summary>
/// <remarks>
/// A checkpoint can only be resumed into a graph with the same executor
/// identities, and identities assigned later do not repair checkpoints written
/// under earlier ones. Logical-role constants, never request or conversation
/// ids, are what keep a resume possible at all.
/// </remarks>
public static class ExecutorIds
{
    public const string OpenRecord = "open-record";
    public const string BuildSlice = "build-slice";
    public const string DraftClosure = "draft-closure";
    public const string ReviewPort = "closure-draft-review";
    public const string HandOff = "hand-off";
}

/// <summary>Opens the act-time record before any work happens.</summary>
/// <remarks>
/// First in the graph on purpose. A record opened after the work would be a
/// record that a crashed run never opens, and the whole point of the
/// write-back leg is that it cannot be skipped.
/// </remarks>
public sealed class OpenRecordExecutor(SpecCli cli)
    : Executor<ImplementRequest, ActRecordOpened>(ExecutorIds.OpenRecord)
{
    private readonly SpecCli _cli = cli;

    public override async ValueTask<ActRecordOpened> HandleAsync(
        ImplementRequest message,
        IWorkflowContext context,
        CancellationToken cancellationToken = default)
    {
        var opened = await _cli.OpenRecordAsync(message, cancellationToken).ConfigureAwait(false);
        await context.AddEventAsync(new RecordOpenedEvent(opened), cancellationToken).ConfigureAwait(false);
        return opened;
    }
}

/// <summary>Builds the slice. This is the half a model may do.</summary>
/// <remarks>
/// The build itself is injected rather than hard-wired, so the graph is
/// testable without a model and so the arrangement — which model, which
/// prompt, which tools — stays a parameter of the experiment rather than a
/// property of the flow.
/// </remarks>
public sealed class BuildSliceExecutor(
    Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> build)
    : Executor<ActRecordOpened, SliceBuilt>(ExecutorIds.BuildSlice)
{
    private readonly Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> _build = build;

    public override ValueTask<SliceBuilt> HandleAsync(
        ActRecordOpened message,
        IWorkflowContext context,
        CancellationToken cancellationToken = default)
        => _build(message, cancellationToken);
}

/// <summary>Turns what was built into the draft a reviewer reads.</summary>
public sealed class DraftClosureExecutor()
    : Executor<SliceBuilt, ClosureDraft>(ExecutorIds.DraftClosure)
{
    public override ValueTask<ClosureDraft> HandleAsync(
        SliceBuilt message,
        IWorkflowContext context,
        CancellationToken cancellationToken = default)
        => new(new ClosureDraft(
            message.RecordId, message.Slice, message.DraftDeterminations, message.Notes));
}

/// <summary>
/// Ends the run by handing the closure to a person.
/// </summary>
/// <remarks>
/// The terminal executor renders the command and stops. It does not run it,
/// and there is no branch in this graph that does: closing is the principal's
/// act, and the graph's shape is what makes that true rather than a rule
/// someone remembered.
/// </remarks>
public sealed class HandOffExecutor()
    : Executor<DraftReview, ImplementOutcome>(ExecutorIds.HandOff)
{
    public override async ValueTask<ImplementOutcome> HandleAsync(
        DraftReview message,
        IWorkflowContext context,
        CancellationToken cancellationToken = default)
    {
        var outcome = new ImplementOutcome(
            message.RecordId,
            message.Determinations,
            SpecCli.HandOffCommand(message.RecordId, message.Determinations));
        await context.YieldOutputAsync(outcome, cancellationToken).ConfigureAwait(false);
        return outcome;
    }
}

/// <summary>Raised once the act-time record exists, so a caller can log it.</summary>
public sealed class RecordOpenedEvent(ActRecordOpened opened) : WorkflowEvent(opened)
{
    public ActRecordOpened Opened { get; } = opened;
}
