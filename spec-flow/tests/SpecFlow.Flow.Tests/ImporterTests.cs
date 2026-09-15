using SpecFlow.Import;

namespace SpecFlow.Flow.Tests;

/// <summary>The scan, over a codebase carrying every shape it recognises.</summary>
public class ImporterTests
{
    [Fact]
    public void Controller_routes_become_entry_points()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        var routes = inventory.EntryPoints.Where(e => e.Kind is "http-route").ToList();
        Assert.Equal(2, routes.Count);
        Assert.Contains(routes, r => r.Transport is "HttpPost /baskets/{id}/settle");
        Assert.Contains(routes, r => r.Symbol is "Shop.Api.BasketController.Settle");
    }

    [Fact]
    public void Minimal_apis_become_entry_points()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        var minimal = inventory.EntryPoints.Where(e => e.Kind is "minimal-api").ToList();
        Assert.Equal(2, minimal.Count);
        Assert.Contains(minimal, m => m.Transport is "MapGet /health");
        Assert.Contains(minimal, m => m.Transport is "MapPost /baskets");
    }

    [Fact]
    public void Hosted_services_and_message_handlers_are_entry_points_too()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.Contains(inventory.EntryPoints, e => e.Kind is "hosted-service"
            && e.Symbol is "Shop.Workers.SettlementWorker");
        Assert.Contains(inventory.EntryPoints, e => e.Kind is "message-handler"
            && e.Transport is "IConsumer<BasketSettled>");
    }

    [Fact]
    public void Build_output_is_not_scanned()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.DoesNotContain(inventory.Symbols, s => s.Namespace is "Shop.Generated");
        Assert.DoesNotContain(inventory.EntryPoints, e => e.Transport.Contains("noise", StringComparison.Ordinal));
    }

    [Fact]
    public void Composition_edges_follow_properties_fields_and_constructors()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.Contains(inventory.CompositionEdges,
            e => e.From is "Shop.Domain.Basket" && e.To is "Money" && e.Via is "property");
        Assert.Contains(inventory.CompositionEdges,
            e => e.From is "Shop.Domain.Basket" && e.To is "Line" && e.Via is "property");
        Assert.Contains(inventory.CompositionEdges,
            e => e.From is "Shop.Api.BasketController" && e.To is "IBasketStore" && e.Via is "ctor-param");
        Assert.Contains(inventory.CompositionEdges,
            e => e.From is "Shop.Api.BasketController" && e.To is "ControllerBase" && e.Via is "base");
    }

    [Fact]
    public void Every_entry_point_yields_a_candidate_with_unfilled_slots()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.Equal(inventory.EntryPoints.Count, inventory.Candidates.Count);
        Assert.All(inventory.Candidates, c =>
        {
            Assert.Equal(["name", "settles"], c.UnfilledSlots);
            Assert.True(c.Observed.ContainsKey("transport"));
        });
    }

    [Fact]
    public void A_candidate_never_guesses_an_act_name()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.All(inventory.Candidates, c => Assert.DoesNotContain("name", c.Observed.Keys));
    }

    [Fact]
    public void Slice_and_realises_fact_attributes_are_scanned_as_claims()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        Assert.Contains(inventory.Claims,
            c => c.Kind is "slice" && c.Value is "checkout-totals"
                 && c.Symbol is "Shop.Domain.Settlement");
        Assert.Contains(inventory.Claims,
            c => c.Kind is "realises-fact" && c.Value is "det/basket-rounding-is-half-even"
                 && c.Symbol is "Shop.Domain.Settlement.Round");
    }

    [Fact]
    public void An_attribute_this_tool_does_not_know_is_not_a_claim()
    {
        using var fixture = new ImportFixture();
        var inventory = Importer.Scan(fixture.Root);

        // The fixture is full of [HttpPost], [ApiController] and friends.
        Assert.All(inventory.Claims, c => Assert.Contains(c.Kind, new[] { "slice", "realises-fact" }));
    }

    [Fact]
    public void An_attribute_with_no_argument_claims_nothing()
    {
        using var fixture = new ImportFixture();
        fixture.Upsert("src/Bare.cs", """
            namespace Shop.Domain;
            [Slice]
            public class Bare { }
            """);
        var inventory = Importer.Scan(fixture.Root);

        Assert.DoesNotContain(inventory.Claims, c => c.Symbol is "Shop.Domain.Bare");
    }

    [Fact]
    public void An_unchanged_codebase_scans_byte_identically()
    {
        using var fixture = new ImportFixture();

        var first = Importer.Scan(fixture.Root) with { ScannedAt = DateTimeOffset.UnixEpoch };
        var second = Importer.Scan(fixture.Root) with { ScannedAt = DateTimeOffset.UnixEpoch };

        Assert.Equal(first.ToJson(), second.ToJson());
    }

    [Fact]
    public void A_new_endpoint_appears_in_the_re_run()
    {
        using var fixture = new ImportFixture();
        var before = Importer.Scan(fixture.Root);

        fixture.Upsert("src/RefundController.cs", """
            namespace Shop.Api;
            public class RefundController : ControllerBase
            {
                [HttpPost("/refunds")]
                public IActionResult Raise() => Ok();
            }
            """);
        var after = Importer.Scan(fixture.Root);

        var appeared = after.EntryPoints.Select(e => e.Id).Except(before.EntryPoints.Select(e => e.Id)).ToList();
        Assert.Single(appeared);
        Assert.Contains("RefundController", appeared[0], StringComparison.Ordinal);
    }

    [Fact]
    public void A_removed_endpoint_disappears_from_the_re_run()
    {
        using var fixture = new ImportFixture();
        var before = Importer.Scan(fixture.Root);

        fixture.Remove("src/SettlementWorker.cs");
        var after = Importer.Scan(fixture.Root);

        var vanished = before.EntryPoints.Select(e => e.Id).Except(after.EntryPoints.Select(e => e.Id)).ToList();
        Assert.Equal(2, vanished.Count);
    }

    [Fact]
    public void Candidate_ids_are_stable_across_an_unrelated_edit()
    {
        using var fixture = new ImportFixture();
        var before = Importer.Scan(fixture.Root).Candidates.Select(c => c.Id).ToList();

        fixture.Upsert("src/Money.cs", """
            namespace Shop.Domain;
            public record Money(decimal Amount, string Currency, string Note);
            """);
        var after = Importer.Scan(fixture.Root).Candidates.Select(c => c.Id).ToList();

        Assert.Equal(before, after);
    }
}
