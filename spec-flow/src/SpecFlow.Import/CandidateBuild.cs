namespace SpecFlow.Import;

/// <summary>Turns observed entry points into candidates awaiting ratification.</summary>
/// <remarks>
/// <para>
/// A candidate is a <em>question</em>, not a proposal. Everything it carries
/// was observed at a transport; nothing it carries names an act. That is why
/// the slots exist and why they start empty: a reviewer supplies the act's
/// name and what it settles, and until they do the candidate says only "the
/// outside world gets in here".
/// </para>
/// <para>
/// The importer deliberately does not guess a name from the route. A guessed
/// name is the failure this whole vocabulary is graded measurement-only to
/// avoid: accept a route's name as an act's name and every act becomes an
/// endpoint with a better label.
/// </para>
/// </remarks>
public static class CandidateBuild
{
    /// <summary>The slots ratification must fill, in the order a reviewer meets them.</summary>
    public static IReadOnlyList<string> Slots { get; } = ["name", "settles"];

    /// <summary>One candidate per entry point.</summary>
    public static List<Candidate> From(IEnumerable<EntryPoint> entryPoints) =>
    [
        .. entryPoints.Select(entry => new Candidate(
            $"cand/{Slug(entry.Id)}",
            entry.Id,
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["kind"] = entry.Kind,
                ["transport"] = entry.Transport,
                ["symbol"] = entry.Symbol,
                ["file"] = entry.File,
                ["line"] = entry.Line.ToString(System.Globalization.CultureInfo.InvariantCulture),
            },
            Slots))
    ];

    /// <summary>
    /// A stable, filesystem-safe id.
    /// </summary>
    /// <remarks>
    /// Stability matters more than prettiness: the id is what the re-run diff
    /// joins on, so a candidate that survives an edit must keep its id.
    /// </remarks>
    public static string Slug(string entryPointId)
    {
        var mapped = entryPointId
            .Select(c => char.IsLetterOrDigit(c) ? char.ToLowerInvariant(c) : '-')
            .ToArray();
        var collapsed = new string(mapped).Split('-', StringSplitOptions.RemoveEmptyEntries);
        return string.Join('-', collapsed);
    }
}
