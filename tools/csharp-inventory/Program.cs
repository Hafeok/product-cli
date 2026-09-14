// csharp-inventory <solution.sln | solution.slnx | project.csproj> [--out <file>]
//
// Loads the solution through MSBuildWorkspace and writes the inventory
// artefact. Exit 0 on a written inventory (workspace diagnostics are carried
// inside it, not hidden), 2 on a usage or load error.

using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Build.Locator;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.MSBuild;

namespace CSharpInventory;

public static class Program
{
    public static async Task<int> Main(string[] args)
    {
        string? input = null;
        string output = "inventory.json";
        for (var i = 0; i < args.Length; i++)
        {
            if (args[i] == "--out" && i + 1 < args.Length) { output = args[++i]; continue; }
            if (args[i].StartsWith("--", StringComparison.Ordinal)) return Usage($"unknown option {args[i]}");
            if (input is not null) return Usage("one solution or project path expected");
            input = args[i];
        }
        if (input is null) return Usage("a solution (.sln/.slnx) or project (.csproj) path is required");
        var inputPath = Path.GetFullPath(input);
        if (!File.Exists(inputPath)) return Usage($"not found: {inputPath}");

        MSBuildLocator.RegisterDefaults();
        var inventory = await Load(inputPath);
        var options = new JsonSerializerOptions
        {
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
            DictionaryKeyPolicy = null,
            WriteIndented = true,
            DefaultIgnoreCondition = JsonIgnoreCondition.Never,
        };
        await File.WriteAllTextAsync(output, JsonSerializer.Serialize(inventory, options) + "\n");
        Console.Error.WriteLine(
            $"inventory: {inventory.Projects.Count} project(s), {inventory.Types.Count} type(s), " +
            $"{inventory.Members.Count} member(s), {inventory.References.Count} reference(s), " +
            $"{inventory.Diagnostics.Count} diagnostic(s) → {output}");
        return 0;
    }

    private static int Usage(string why)
    {
        Console.Error.WriteLine($"csharp-inventory: {why}");
        Console.Error.WriteLine("usage: csharp-inventory <solution.sln | solution.slnx | project.csproj> [--out <file>]");
        return 2;
    }

    // Kept separate from Main so the workspace is disposed before the JSON
    // is written; MSBuildWorkspace holds project files open until then.
    private static async Task<Inventory> Load(string inputPath)
    {
        var diagnostics = new List<Diagnostic>();
        using var workspace = MSBuildWorkspace.Create();
        workspace.RegisterWorkspaceFailedHandler(e => diagnostics.Add(new Diagnostic
        {
            Severity = e.Diagnostic.Kind == WorkspaceDiagnosticKind.Failure ? "error" : "warning",
            Message = e.Diagnostic.Message,
        }));

        var solutionDir = Path.GetDirectoryName(inputPath) ?? ".";
        var isSolution = inputPath.EndsWith(".sln", StringComparison.OrdinalIgnoreCase) || inputPath.EndsWith(".slnx", StringComparison.OrdinalIgnoreCase);
        var projects = isSolution
            ? (await workspace.OpenSolutionAsync(inputPath)).Projects.ToList()
            : new List<Project> { await workspace.OpenProjectAsync(inputPath) };

        var collector = new Collector(solutionDir, diagnostics);
        await Collector.FindAmbiguous(projects);
        foreach (var project in projects.OrderBy(p => p.Name, StringComparer.Ordinal))
            await collector.AddProject(project);

        return collector.Finish(new SolutionInfo
        {
            Path = Collector.Relative(solutionDir, inputPath),
            GitHead = GitHead(solutionDir),
        });
    }

    private static string? GitHead(string dir)
    {
        try
        {
            var psi = new ProcessStartInfo("git", "rev-parse HEAD")
            {
                WorkingDirectory = dir, RedirectStandardOutput = true, RedirectStandardError = true,
            };
            using var p = Process.Start(psi);
            if (p is null) return null;
            var head = p.StandardOutput.ReadToEnd().Trim();
            p.WaitForExit();
            return p.ExitCode == 0 && head.Length == 40 ? head : null;
        }
        catch (Exception)
        {
            return null;
        }
    }
}
