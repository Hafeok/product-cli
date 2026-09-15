namespace SpecFlow.Flow.Tests;

/// <summary>
/// The implement workflow end to end, against the real Rust store.
/// </summary>
public class ImplementWorkflowTests
{
    private static ValueTask<SliceBuilt> BuiltWith(ActRecordOpened opened, params string[] determinations)
        => new(new SliceBuilt(opened.RecordId, opened.Slice, determinations, "built in a test"));

    private static ImplementDriver Driver(
        SpecRepo repo,
        string[] drafted,
        Func<ClosureDraft, DraftReview> review)
        => new(
            repo.Cli(),
            (opened, _) => BuiltWith(opened, drafted),
            (draft, _) => new ValueTask<DraftReview>(review(draft)));

    private static ImplementRequest Request { get; } =
        new("checkout-totals", "act/settle-basket", "agent@example.invalid");

    [Fact]
    public async Task A_run_opens_a_record_and_hands_the_closure_over()
    {
        using var repo = new SpecRepo();
        var driver = Driver(repo, ["det/basket-rounding"], d => new DraftReview(d.RecordId, d.DraftDeterminations));

        var outcome = await driver.RunAsync(Request);

        Assert.NotEmpty(outcome.RecordId);
        Assert.True(outcome.ClosurePending);
        Assert.Contains($"spec close {outcome.RecordId}", outcome.HandOffCommand, StringComparison.Ordinal);
    }

    [Fact]
    public async Task The_gate_still_fails_after_an_unattended_run()
    {
        using var repo = new SpecRepo();
        var driver = Driver(repo, [], d => new DraftReview(d.RecordId, []));

        await driver.RunAsync(Request);

        Assert.Equal(1, repo.Check());
    }

    [Fact]
    public async Task The_reviewer_sees_the_draft_and_can_amend_it()
    {
        using var repo = new SpecRepo();
        ClosureDraft? seen = null;
        var driver = Driver(
            repo,
            ["det/drafted-by-the-model"],
            d =>
            {
                seen = d;
                return new DraftReview(d.RecordId, ["det/what-the-human-actually-decided"]);
            });

        var outcome = await driver.RunAsync(Request);

        Assert.NotNull(seen);
        Assert.Equal("checkout-totals", seen.Slice);
        Assert.Equal(["det/drafted-by-the-model"], seen.DraftDeterminations);
        Assert.Equal(["det/what-the-human-actually-decided"], outcome.ReviewedDeterminations);
    }

    [Fact]
    public async Task An_empty_review_hands_over_a_nothing_arose_command()
    {
        using var repo = new SpecRepo();
        var driver = Driver(repo, ["det/drafted"], d => new DraftReview(d.RecordId, []));

        var outcome = await driver.RunAsync(Request);

        Assert.Contains("--nothing-arose", outcome.HandOffCommand, StringComparison.Ordinal);
    }

    [Fact]
    public async Task The_record_is_opened_before_the_slice_is_built()
    {
        using var repo = new SpecRepo();
        var openWhenBuildRan = -1;
        var driver = new ImplementDriver(
            repo.Cli(),
            (opened, _) =>
            {
                openWhenBuildRan = repo.Check();
                return BuiltWith(opened);
            },
            (draft, _) => new ValueTask<DraftReview>(new DraftReview(draft.RecordId, [])));

        await driver.RunAsync(Request);

        Assert.Equal(1, openWhenBuildRan);
    }
}
