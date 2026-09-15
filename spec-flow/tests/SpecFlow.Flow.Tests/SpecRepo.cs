using System.Diagnostics;

namespace SpecFlow.Flow.Tests;

/// <summary>A throwaway git repo with the `spec` binary pointed at it.</summary>
public sealed class SpecRepo : IDisposable
{
    public string Root { get; }
    public string Executable { get; }

    public SpecRepo()
    {
        Root = Path.Combine(Path.GetTempPath(), "specflow-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Root);
        Git("init", "-q", ".");
        Git("config", "user.email", "emil@example.com");
        Git("config", "user.name", "Emil");
        Git("commit", "-q", "--allow-empty", "-m", "init");
        Executable = LocateSpec();
    }

    public SpecCli Cli() => new(Executable, Root);

    /// <summary>Run the binary directly, for asserting on the gate.</summary>
    public int Check()
    {
        var info = new ProcessStartInfo(Executable) { RedirectStandardOutput = true, RedirectStandardError = true };
        info.ArgumentList.Add("--root");
        info.ArgumentList.Add(Root);
        info.ArgumentList.Add("check");
        using var process = Process.Start(info)!;
        process.WaitForExit();
        return process.ExitCode;
    }

    /// <summary>
    /// The debug build of the Rust CLI. `SPEC_BIN` overrides, so CI can point
    /// at a release artifact without the test knowing where it landed.
    /// </summary>
    private static string LocateSpec()
    {
        var supplied = Environment.GetEnvironmentVariable("SPEC_BIN");
        if (!string.IsNullOrWhiteSpace(supplied))
        {
            return supplied;
        }
        var dir = AppContext.BaseDirectory;
        for (var probe = new DirectoryInfo(dir); probe is not null; probe = probe.Parent)
        {
            var candidate = Path.Combine(probe.FullName, "target", "debug", "spec");
            if (File.Exists(candidate))
            {
                return candidate;
            }
        }
        throw new InvalidOperationException(
            "could not find the `spec` binary; build it with `cargo build -p spec-cli` or set SPEC_BIN");
    }

    private void Git(params string[] args)
    {
        var info = new ProcessStartInfo("git") { WorkingDirectory = Root };
        foreach (var arg in args)
        {
            info.ArgumentList.Add(arg);
        }
        using var process = Process.Start(info)!;
        process.WaitForExit();
    }

    public void Dispose()
    {
        try
        {
            Directory.Delete(Root, recursive: true);
        }
        catch (IOException)
        {
            // A leftover temp directory is not worth failing a test over.
        }
    }
}
