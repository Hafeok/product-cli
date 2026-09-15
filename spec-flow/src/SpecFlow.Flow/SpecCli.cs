using System.Diagnostics;
using System.Text;

namespace SpecFlow.Flow;

/// <summary>
/// The only way this assembly touches the record store: by invoking the Rust
/// <c>spec</c> binary.
/// </summary>
/// <remarks>
/// <para>
/// The store has one writer, and it is not this process. That is not an
/// ergonomic choice — it is where the accountability boundary of the
/// specification flow falls. A model may drive everything up to the decision;
/// it may not commit it.
/// </para>
/// <para>
/// <see cref="ForbiddenVerbs"/> makes the restriction structural rather than
/// instructed. An agent host told not to call a verb is prose; a host that
/// throws when the verb is assembled is a boundary.
/// </para>
/// </remarks>
public sealed class SpecCli(string executable, string repoRoot)
{
    /// <summary>
    /// Verbs this process may never invoke, whatever asks it to.
    /// </summary>
    public static readonly IReadOnlySet<string> ForbiddenVerbs =
        new HashSet<string>(StringComparer.Ordinal) { "close" };

    private readonly string _executable = executable;
    private readonly string _repoRoot = repoRoot;

    /// <summary>The result of one CLI invocation.</summary>
    public sealed record Result(int ExitCode, string Stdout, string Stderr);

    /// <summary>
    /// Run the <c>spec</c> binary, refusing any verb on the forbidden list.
    /// </summary>
    /// <exception cref="InvalidOperationException">
    /// The arguments name a verb this process may not call.
    /// </exception>
    public async Task<Result> RunAsync(
        IReadOnlyList<string> arguments,
        CancellationToken cancellationToken = default)
    {
        Guard(arguments);

        var info = new ProcessStartInfo(_executable)
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
        };
        info.ArgumentList.Add("--root");
        info.ArgumentList.Add(_repoRoot);
        foreach (var argument in arguments)
        {
            info.ArgumentList.Add(argument);
        }

        using var process = Process.Start(info)
            ?? throw new InvalidOperationException($"could not start `{_executable}`");

        var stdout = process.StandardOutput.ReadToEndAsync(cancellationToken);
        var stderr = process.StandardError.ReadToEndAsync(cancellationToken);
        await process.WaitForExitAsync(cancellationToken).ConfigureAwait(false);
        return new Result(process.ExitCode, await stdout.ConfigureAwait(false), await stderr.ConfigureAwait(false));
    }

    /// <summary>
    /// Throw if the arguments name a verb this process may not call.
    /// </summary>
    public static void Guard(IReadOnlyList<string> arguments)
    {
        foreach (var argument in arguments)
        {
            if (ForbiddenVerbs.Contains(argument))
            {
                throw new InvalidOperationException(
                    $"`spec {argument}` names a principal; an agent host cannot be one. " +
                    "Hand the command to a person and let them run it.");
            }
        }
    }

    /// <summary>
    /// Open an act-time record for a slice. Exit code 3 is the success case:
    /// the record opened and its closure is pending, which is what every
    /// unattended run produces.
    /// </summary>
    public async Task<ActRecordOpened> OpenRecordAsync(
        ImplementRequest request,
        CancellationToken cancellationToken = default)
    {
        var result = await RunAsync(
            ["implement", "--slice", request.Slice, "--act", request.ActRef, "--by", request.OpenedBy, "--json"],
            cancellationToken).ConfigureAwait(false);

        if (result.ExitCode is not 3)
        {
            throw new InvalidOperationException(
                $"`spec implement` exited {result.ExitCode}, expected 3 (closure pending): {result.Stderr}{result.Stdout}");
        }

        var recordId = ReadJsonString(result.Stdout, "record")
            ?? throw new InvalidOperationException($"`spec implement` named no record: {result.Stdout}");
        return new ActRecordOpened(recordId, request.Slice, request.ActRef);
    }

    private static string? ReadJsonString(string json, string property)
    {
        using var document = System.Text.Json.JsonDocument.Parse(json);
        return document.RootElement.TryGetProperty(property, out var value)
            ? value.GetString()
            : null;
    }

    /// <summary>
    /// The command a principal runs to close a record — rendered for a human
    /// to read, run, and answer for. Rendering it is not running it.
    /// </summary>
    public static string HandOffCommand(string recordId, IReadOnlyList<string> determinations)
    {
        var builder = new StringBuilder($"spec close {recordId} --principal <you@example.com>");
        if (determinations.Count is 0)
        {
            builder.Append(" --nothing-arose");
        }
        foreach (var determination in determinations)
        {
            builder.Append(" --determination ").Append(determination);
        }
        return builder.ToString();
    }
}
