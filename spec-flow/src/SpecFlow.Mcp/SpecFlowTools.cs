using System.ComponentModel;
using System.Text.Json;
using ModelContextProtocol.Server;
using SpecFlow.Flow;
using SpecFlow.Import;

namespace SpecFlow.Mcp;

/// <summary>
/// The specification flow's MCP surface — a strict subset, not a mirror.
/// </summary>
/// <remarks>
/// <para>
/// A model may do everything up to the decision. <c>accept</c>, <c>reject</c>,
/// <c>close</c> and <c>policy set</c> each name a principal, so none of them
/// is offered here and none of them could be added: every call out to the
/// store goes through <see cref="SpecCli"/>, which throws on a withheld verb,
/// and this assembly contains no other way to write one.
/// </para>
/// <para>
/// Reads are proxied to the Rust binary rather than reimplemented. One
/// implementation of what the store means is what keeps an MCP client and a CI
/// run from being told different things about the same repo.
/// </para>
/// </remarks>
[McpServerToolType]
public sealed class SpecFlowTools(SpecFlowOptions options)
{
    private readonly SpecFlowOptions _options = options;

    private SpecCli Cli => new(_options.SpecBinary, _options.Root);

    [McpServerTool(Name = "spec_import", Title = "Re-scan the codebase", Idempotent = true)]
    [Description("""
        Scan a C# codebase and rewrite .spec/inventory.json: symbols, composition
        edges, entry points, and one candidate per entry point. Re-runnable, and
        the re-run is the point — the diff against ratified acts is the signal.
        Candidates carry only what was observed at the transport; they never name
        an act, and filling `name` and `settles` is a principal's job at the CLI.
        """)]
    public string Import()
    {
        var inventory = Importer.Scan(_options.ScanRoot);
        var path = Path.Combine(_options.Root, Importer.InventoryPath);
        Directory.CreateDirectory(Path.GetDirectoryName(path) ?? ".");
        File.WriteAllText(path, (inventory with { Root = _options.ScanRoot }).ToJson());

        return JsonSerializer.Serialize(new
        {
            revision = inventory.Revision,
            symbols = inventory.Symbols.Count,
            composition_edges = inventory.CompositionEdges.Count,
            entry_points = inventory.EntryPoints.Count,
            candidates = inventory.Candidates.Count,
            inventory = path,
            note = "The candidate list is not a domain model. Do not author the event model "
                 + "by walking it: the vocabulary is transport-shaped, and an act named after "
                 + "a route is an endpoint with a better label.",
        });
    }

    [McpServerTool(Name = "spec_candidates", Title = "List candidates", ReadOnly = true)]
    [Description("""
        List import candidates with what was observed at the transport and which
        slots ratification must still fill. A candidate is a question, not a
        proposal. Draft a name and what it settles if asked — then hand them to a
        person; do not claim to have filed anything.
        """)]
    public async Task<JsonElement> CandidatesAsync(
        [Description("Only candidates nobody has ratified or refused.")] bool unreviewed = false,
        CancellationToken cancellationToken = default)
        => await Cli.ReadJsonAsync(
            unreviewed ? ["candidates", "--unreviewed"] : ["candidates"],
            cancellationToken).ConfigureAwait(false);

    [McpServerTool(Name = "spec_map", Title = "Join acts to entry points", ReadOnly = true)]
    [Description("""
        Join ratified acts to entry points and report the disagreements: merge
        (several entry points, one act), split (one entry point, several acts),
        unmapped entry point, unmapped act. A restructuring work list, not
        verdicts — each item justified by a named act.
        """)]
    public async Task<JsonElement> MapAsync(CancellationToken cancellationToken = default)
        => await Cli.ReadJsonAsync(["map"], cancellationToken).ConfigureAwait(false);

    [McpServerTool(Name = "spec_check", Title = "Run the gate", ReadOnly = true)]
    [Description("""
        Run the gate. Returns structural verdicts and the project's own policy
        verdicts apart from one another, plus the reported metrics. Structural
        verdicts are not a matter of project policy; the metrics are reported and
        gate nothing unless a filed policy says so.
        """)]
    public async Task<JsonElement> CheckAsync(CancellationToken cancellationToken = default)
        => await Cli.ReadJsonAsync(["check"], cancellationToken).ConfigureAwait(false);

    [McpServerTool(Name = "spec_records", Title = "List act-time records", ReadOnly = true)]
    [Description("List act-time records and whether each is still open.")]
    public async Task<JsonElement> RecordsAsync(
        [Description("Only records still open.")] bool open = false,
        CancellationToken cancellationToken = default)
        => await Cli.ReadJsonAsync(
            open ? ["records", "--open"] : ["records"],
            cancellationToken).ConfigureAwait(false);

    [McpServerTool(Name = "spec_policy_show", Title = "Show the policy in force", ReadOnly = true)]
    [Description("""
        Show the check policy in force: what it gates, on whose word, with what
        basis, and what it reports and deliberately does not gate. With no policy
        filed the answer is structural verdicts only.
        """)]
    public async Task<JsonElement> PolicyShowAsync(CancellationToken cancellationToken = default)
        => await Cli.ReadJsonAsync(["policy", "show"], cancellationToken).ConfigureAwait(false);

    [McpServerTool(Name = "spec_implement", Title = "Build a slice", Destructive = false)]
    [Description("""
        Build a slice against the specification and open its act-time record.
        Produces a PENDING record: closing it names a principal and cannot be done
        from here, or from any tool this server offers. Returns the command a
        person runs to close it. Report the record as open — never as closed,
        accepted, or filed.
        """)]
    public async Task<string> ImplementAsync(
        [Description("The slice being built.")] string slice,
        [Description("The ratified act it realises, e.g. act/settle-a-basket.")] string actRef,
        [Description("Who opens it. May be a machine — building is the delegable half.")]
        string by = "agent@example.invalid",
        CancellationToken cancellationToken = default)
    {
        var driver = new ImplementDriver(Cli, NothingDrafted, KeepTheDraft);
        var outcome = await driver
            .RunAsync(new ImplementRequest(slice, actRef, by), cancellationToken)
            .ConfigureAwait(false);

        return JsonSerializer.Serialize(new
        {
            record = outcome.RecordId,
            status = "pending-closure",
            determinations = outcome.ReviewedDeterminations,
            hand_off = outcome.HandOffCommand,
            note = "The record is OPEN and the gate will fail while it is. Closing names a "
                 + "principal and is not yours to do — give the hand_off command to a person.",
        });
    }

    /// <summary>
    /// The build step, over MCP.
    /// </summary>
    /// <remarks>
    /// The caller is the model, so there is no second model to draft for it:
    /// what arose is the caller's to state, and it states it by running the
    /// hand-off command past a person. The workflow still opens the record
    /// first, which is the part that must not be skippable.
    /// </remarks>
    private static ValueTask<SliceBuilt> NothingDrafted(
        ActRecordOpened opened,
        CancellationToken cancellationToken)
        => new(new SliceBuilt(opened.RecordId, opened.Slice, [], "opened over MCP by the calling agent"));

    /// <summary>
    /// The review port, with no terminal to ask at.
    /// </summary>
    /// <remarks>
    /// Passing the draft through is not the human review being skipped — it is
    /// the review happening later, at `spec close`, under a principal's own
    /// identity. Nothing here files a determination.
    /// </remarks>
    private static ValueTask<DraftReview> KeepTheDraft(
        ClosureDraft draft,
        CancellationToken cancellationToken)
        => new(new DraftReview(draft.RecordId, draft.DraftDeterminations));
}
