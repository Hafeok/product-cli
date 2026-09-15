using Microsoft.Agents.AI.Workflows;

namespace SpecFlow.Flow;

/// <summary>
/// Runs the implement workflow, pumping its event stream and answering the
/// review port.
/// </summary>
/// <remarks>
/// The reviewer callback is the only place a person enters the run. It amends
/// a draft; it never closes a record, because there is nothing here that
/// could. The run's exit code is <see cref="PendingClosure"/> on success,
/// which is the flow's way of saying the machine finished its half.
/// </remarks>
public sealed class ImplementDriver(
    SpecCli cli,
    Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> build,
    Func<ClosureDraft, CancellationToken, ValueTask<DraftReview>> review)
{
    /// <summary>Work completed, closure pending. Not success.</summary>
    public const int PendingClosure = 3;

    private readonly SpecCli _cli = cli;
    private readonly Func<ActRecordOpened, CancellationToken, ValueTask<SliceBuilt>> _build = build;
    private readonly Func<ClosureDraft, CancellationToken, ValueTask<DraftReview>> _review = review;

    /// <summary>Run one implement act to its hand-off.</summary>
    public async Task<ImplementOutcome> RunAsync(
        ImplementRequest request,
        CancellationToken cancellationToken = default)
    {
        var workflow = ImplementWorkflow.Build(_cli, _build);
        await using var run = await InProcessExecution
            .RunStreamingAsync(workflow, request, cancellationToken: cancellationToken)
            .ConfigureAwait(false);

        ImplementOutcome? outcome = null;
        await foreach (var evt in run.WatchStreamAsync(cancellationToken).ConfigureAwait(false))
        {
            switch (evt)
            {
                case RequestInfoEvent request_ when request_.Request.TryGetDataAs<ClosureDraft>(out var draft)
                                                    && draft is not null:
                    var reviewed = await _review(draft, cancellationToken).ConfigureAwait(false);
                    await run.SendResponseAsync(request_.Request.CreateResponse(reviewed)).ConfigureAwait(false);
                    break;

                case WorkflowOutputEvent output when output.As<ImplementOutcome>() is { } produced:
                    outcome = produced;
                    break;

                case WorkflowErrorEvent error:
                    throw new InvalidOperationException(
                        $"the implement workflow failed: {error.Data}");
            }
        }

        return outcome ?? throw new InvalidOperationException(
            "the implement workflow produced no outcome — the record may be open with no hand-off");
    }
}
