using System.Text.Json;
using System.Text.Json.Serialization;

namespace SpecFlow.Import;

/// <summary>A declaration the scan found, addressed by a stable id.</summary>
/// <param name="Id">
/// <c>&lt;namespace&gt;.&lt;containing types&gt;.&lt;name&gt;</c>. Stable across
/// a move between files, which is what makes the re-run diff mean something.
/// </param>
public sealed record SymbolEntry(
    string Id,
    string Kind,
    string Name,
    string Namespace,
    string File,
    int Line);

/// <summary>
/// One type reaching another: a field, a property, a constructor parameter, or
/// a base type.
/// </summary>
public sealed record CompositionEdge(string From, string To, string Via);

/// <summary>
/// A place the outside world gets in.
/// </summary>
/// <param name="Kind">
/// <c>http-route</c>, <c>minimal-api</c>, <c>hosted-service</c>,
/// <c>message-handler</c> or <c>cli-entry</c>.
/// </param>
/// <param name="Transport">
/// The transport-shaped address — a route template, a queue name, a method
/// name. Recorded because it is observed, never because it names an act.
/// </param>
public sealed record EntryPoint(
    string Id,
    string Kind,
    string Symbol,
    string Transport,
    string File,
    int Line);

/// <summary>
/// A candidate act: an entry point with the slots ratification must fill.
/// </summary>
/// <remarks>
/// The observed fields are transport-shaped by construction, which is why the
/// vocabulary is graded measurement-only and why <c>model</c> never displays
/// this list. The unfilled slots are the point: a candidate is a question, not
/// a proposal.
/// </remarks>
public sealed record Candidate(
    string Id,
    string EntryPoint,
    IReadOnlyDictionary<string, string> Observed,
    IReadOnlyList<string> UnfilledSlots);

/// <summary>
/// What one <c>import</c> run observed.
/// </summary>
/// <remarks>
/// A <em>projection</em> of the codebase at a revision, never authority. It is
/// rebuilt wholesale on every run and carries no verdict, no ratification and
/// no decision — those live in the record store, which this process cannot
/// write. Deleting this file loses nothing a re-run does not restore.
/// </remarks>
public sealed record Inventory(
    string Form,
    string Root,
    string Revision,
    DateTimeOffset ScannedAt,
    IReadOnlyList<SymbolEntry> Symbols,
    IReadOnlyList<CompositionEdge> CompositionEdges,
    IReadOnlyList<EntryPoint> EntryPoints,
    IReadOnlyList<Candidate> Candidates)
{
    public const string FormV1 = "spec.inventory.v1";

    /// <summary>
    /// Snake-case on the wire, because the reader is Rust.
    /// </summary>
    /// <remarks>
    /// The inventory crosses a runtime boundary, so its field naming is a
    /// contract rather than a style preference. Snake case is what the record
    /// store on the other side already speaks.
    /// </remarks>
    private static readonly JsonSerializerOptions Options = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    };

    /// <summary>
    /// Serialise, with every list ordered by id.
    /// </summary>
    /// <remarks>
    /// Byte-stability is what makes the re-run diff readable: an unchanged
    /// codebase must produce an unchanged file, or every run looks like drift.
    /// </remarks>
    public string ToJson() => JsonSerializer.Serialize(Canonical(), Options);

    /// <summary>The same inventory with a deterministic order.</summary>
    public Inventory Canonical() => this with
    {
        Symbols = [.. Symbols.OrderBy(s => s.Id, StringComparer.Ordinal)],
        CompositionEdges = [.. CompositionEdges
            .OrderBy(e => e.From, StringComparer.Ordinal)
            .ThenBy(e => e.To, StringComparer.Ordinal)
            .ThenBy(e => e.Via, StringComparer.Ordinal)],
        EntryPoints = [.. EntryPoints.OrderBy(e => e.Id, StringComparer.Ordinal)],
        Candidates = [.. Candidates.OrderBy(c => c.Id, StringComparer.Ordinal)],
    };
}
