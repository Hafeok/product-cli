namespace SpecFlow.Flow;

/// <summary>What the caller asks the implement workflow to do.</summary>
/// <param name="Slice">The slice to build against the specification.</param>
/// <param name="ActRef">The specification act the slice realises.</param>
/// <param name="OpenedBy">
/// Who opens the record. May be a machine — building is the delegable half.
/// </param>
public sealed record ImplementRequest(string Slice, string ActRef, string OpenedBy);

/// <summary>An act-time record the Rust CLI opened, named by its id.</summary>
public sealed record ActRecordOpened(string RecordId, string Slice, string ActRef);

/// <summary>
/// The slice as built, with the determinations acting produced. Drafted, never
/// filed: a determination becomes real when a principal closes the record.
/// </summary>
public sealed record SliceBuilt(
    string RecordId,
    string Slice,
    IReadOnlyList<string> DraftDeterminations,
    string Notes);

/// <summary>
/// What the workflow puts to a reviewer. A draft is shown for amendment, not
/// for approval — nothing here closes anything.
/// </summary>
public sealed record ClosureDraft(
    string RecordId,
    string Slice,
    IReadOnlyList<string> DraftDeterminations,
    string Notes);

/// <summary>
/// A reviewer's amendment of the draft. Still a draft: the reviewer's decision
/// is taken later, at the CLI, under their own identity.
/// </summary>
/// <param name="Determinations">
/// The determination addresses as the reviewer would file them. Empty means
/// they expect to close with <c>--nothing-arose</c>.
/// </param>
public sealed record DraftReview(string RecordId, IReadOnlyList<string> Determinations);

/// <summary>
/// What an unattended run produces: a built slice, an open record, and the
/// command a principal runs to close it. Never a closure.
/// </summary>
public sealed record ImplementOutcome(
    string RecordId,
    IReadOnlyList<string> ReviewedDeterminations,
    string HandOffCommand)
{
    /// <summary>
    /// Always true for this workflow. The type carries it so a caller reading
    /// the outcome cannot mistake a completed run for a closed record.
    /// </summary>
    public bool ClosurePending => true;
}
