namespace SpecFlow.Cli;

/// <summary>The flow's exit codes, matching the Rust half.</summary>
internal static class ExitCodes
{
    public const int Conformant = 0;
    public const int Findings = 1;
    public const int CouldNotRun = 2;

    /// <summary>Work completed, closure pending. Not success.</summary>
    public const int PendingClosure = 3;
}
