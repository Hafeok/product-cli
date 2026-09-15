using System.Diagnostics;
using Microsoft.CodeAnalysis.CSharp;

namespace SpecFlow.Import;

/// <summary>Reads a C# codebase and writes the inventory.</summary>
/// <remarks>
/// Re-runnable by construction, and the re-run is the point: endpoints appear,
/// candidates regenerate, the delta moves. An importer used once at onboarding
/// answers a question nobody asks twice.
/// </remarks>
public static class Importer
{
    /// <summary>Where the inventory lands, relative to the repo root.</summary>
    public const string InventoryPath = ".spec/inventory.json";

    /// <summary>Scan a tree of C# sources.</summary>
    public static Inventory Scan(string root)
    {
        var symbols = new List<SymbolEntry>();
        var edges = new List<CompositionEdge>();
        var entryPoints = new List<EntryPoint>();

        foreach (var file in SourceFiles(root))
        {
            var relative = Path.GetRelativePath(root, file).Replace('\\', '/');
            var tree = CSharpSyntaxTree.ParseText(File.ReadAllText(file), path: relative);
            var (fileSymbols, fileEdges) = SymbolScan.Scan(tree, relative);
            symbols.AddRange(fileSymbols);
            edges.AddRange(fileEdges);
            entryPoints.AddRange(EntryPointScan.Scan(tree, relative));
        }

        return new Inventory(
            Inventory.FormV1,
            root,
            HeadRevision(root),
            DateTimeOffset.UtcNow,
            symbols,
            edges,
            entryPoints,
            CandidateBuild.From(entryPoints)).Canonical();
    }

    /// <summary>Scan and write, returning where it landed.</summary>
    public static string Write(string root)
    {
        var inventory = Scan(root);
        var path = Path.Combine(root, InventoryPath);
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        File.WriteAllText(path, inventory.ToJson());
        return path;
    }

    /// <summary>
    /// Every C# source under the root, skipping build output.
    /// </summary>
    /// <remarks>
    /// Generated and vendored trees are excluded because an act is something a
    /// team decided, and nobody decided <c>obj/Debug/…</c>.
    /// </remarks>
    public static IEnumerable<string> SourceFiles(string root)
    {
        string[] skipped = ["bin", "obj", "node_modules", ".git", "target", "TestResults"];
        return Directory
            .EnumerateFiles(root, "*.cs", SearchOption.AllDirectories)
            .Where(f => !Path.GetRelativePath(root, f)
                .Split(Path.DirectorySeparatorChar, '/')
                .Any(segment => skipped.Contains(segment, StringComparer.Ordinal)))
            .Where(f => !f.EndsWith(".g.cs", StringComparison.Ordinal))
            .OrderBy(f => f, StringComparer.Ordinal);
    }

    /// <summary>The revision scanned, or <c>unknown</c> outside a git tree.</summary>
    public static string HeadRevision(string root)
    {
        try
        {
            var info = new ProcessStartInfo("git")
            {
                WorkingDirectory = root,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            };
            info.ArgumentList.Add("rev-parse");
            info.ArgumentList.Add("HEAD");
            using var process = Process.Start(info);
            if (process is null)
            {
                return "unknown";
            }
            var output = process.StandardOutput.ReadToEnd().Trim();
            process.StandardError.ReadToEnd();
            process.WaitForExit();
            return process.ExitCode is 0 && output.Length > 0 ? output : "unknown";
        }
        catch (Exception e) when (e is System.ComponentModel.Win32Exception or IOException)
        {
            return "unknown";
        }
    }
}
