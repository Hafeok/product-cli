namespace SpecFlow.Flow.Tests;

/// <summary>A throwaway C# codebase with the shapes the scanner recognises.</summary>
public sealed class ImportFixture : IDisposable
{
    public string Root { get; }

    public ImportFixture()
    {
        Root = Path.Combine(Path.GetTempPath(), "specimport-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Path.Combine(Root, "src"));
        Directory.CreateDirectory(Path.Combine(Root, "obj", "Debug"));

        Write("src/BasketController.cs", """
            using Microsoft.AspNetCore.Mvc;

            namespace Shop.Api;

            [ApiController]
            public class BasketController : ControllerBase
            {
                private readonly IBasketStore _store;
                public BasketController(IBasketStore store) => _store = store;

                [HttpPost("/baskets/{id}/settle")]
                public IActionResult Settle(string id) => Ok();

                [HttpGet("/baskets/{id}")]
                public IActionResult Read(string id) => Ok();
            }
            """);

        Write("src/Money.cs", """
            namespace Shop.Domain;

            public record Money(decimal Amount, string Currency);

            public class Basket
            {
                public Money Total { get; set; } = new(0, "DKK");
                public List<Line> Lines { get; } = new();
            }

            public record Line(string Sku, Money Price);
            """);

        Write("src/SettlementWorker.cs", """
            namespace Shop.Workers;

            public class SettlementWorker : BackgroundService
            {
                protected override Task ExecuteAsync(CancellationToken stoppingToken) => Task.CompletedTask;
            }

            public class BasketSettledConsumer : IConsumer<BasketSettled>
            {
                public Task Consume(ConsumeContext<BasketSettled> context) => Task.CompletedTask;
            }
            """);

        Write("src/Program.cs", """
            var app = WebApplication.CreateBuilder(args).Build();
            app.MapGet("/health", () => "ok");
            app.MapPost("/baskets", () => Results.Created());
            app.Run();
            """);

        // Build output must not become candidates: nobody decided obj/Debug.
        Write("obj/Debug/Generated.cs", """
            namespace Shop.Generated;
            public class Noise
            {
                [HttpGet("/noise")]
                public void Go() { }
            }
            """);
    }

    private void Write(string relative, string content)
    {
        var path = Path.Combine(Root, relative);
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        File.WriteAllText(path, content);
    }

    /// <summary>Add or replace a file, for exercising the re-run diff.</summary>
    public void Upsert(string relative, string content) => Write(relative, content);

    public void Remove(string relative) => File.Delete(Path.Combine(Root, relative));

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
