using SpecFlow.Import;

namespace SpecFlow.Cli;

/// <summary>The <c>import</c> verb: codebase in, inventory out.</summary>
/// <remarks>
/// Deterministic and model-free. Nothing here proposes an act, and nothing
/// here ratifies one — the output is an observation of where the outside world
/// gets in, which is the only thing a codebase can honestly tell you.
/// </remarks>
internal static class ImportCommand
{
    public static int Run(Options options)
    {
        var source = options.Get("source") ?? options.Root;
        if (!Directory.Exists(source))
        {
            Console.Error.WriteLine($"no such directory: {source}");
            return ExitCodes.CouldNotRun;
        }

        var inventory = Importer.Scan(source);
        var path = Path.Combine(options.Root, Importer.InventoryPath);
        Directory.CreateDirectory(Path.GetDirectoryName(path) ?? ".");
        File.WriteAllText(path, (inventory with { Root = source }).ToJson());

        Console.WriteLine($"scanned {source} at {inventory.Revision}");
        Console.WriteLine(
            $"  {inventory.Symbols.Count} symbol(s), {inventory.CompositionEdges.Count} edge(s), " +
            $"{inventory.EntryPoints.Count} entry point(s), {inventory.Candidates.Count} candidate(s)");
        Console.WriteLine($"  → {path}");
        Console.WriteLine();
        Console.WriteLine("The candidate list is not a domain model. Review it with `spec candidates`,");
        Console.WriteLine("and ratify one at a time with `spec accept`, which names a principal.");
        return ExitCodes.Conformant;
    }
}
