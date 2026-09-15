namespace SpecFlow.Cli;

/// <summary>
/// The agent host: the delegable half of the specification flow.
/// </summary>
/// <remarks>
/// It imports codebases, builds slices and hands closures over. It cannot
/// ratify a candidate or close a record — not by policy, but because nothing
/// in the assemblies it links offers either verb.
/// </remarks>
internal static class Program
{
    private static async Task<int> Main(string[] args)
    {
        var options = Options.Parse(args);
        if (options is null)
        {
            Console.Error.WriteLine(Options.Usage);
            return ExitCodes.CouldNotRun;
        }

        try
        {
            return options.Verb switch
            {
                "import" => ImportCommand.Run(options),
                "implement" => await ImplementCommand.RunAsync(options).ConfigureAwait(false),
                "mcp" => await McpCommand.RunAsync(options).ConfigureAwait(false),
                _ => Unknown(options.Verb),
            };
        }
        catch (Exception e) when (e is InvalidOperationException or IOException)
        {
            Console.Error.WriteLine(e.Message);
            return ExitCodes.CouldNotRun;
        }
    }

    private static int Unknown(string verb)
    {
        Console.Error.WriteLine($"unknown verb `{verb}`\n");
        Console.Error.WriteLine(Options.Usage);
        return ExitCodes.CouldNotRun;
    }
}
