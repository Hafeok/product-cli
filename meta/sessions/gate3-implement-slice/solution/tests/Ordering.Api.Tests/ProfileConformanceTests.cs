namespace Ordering.Api.Tests;

using System.Reflection;
using Ordering.Api.Profile;
using Ordering.Api.Slices.PlaceOrder;
using Ordering.Api.Unroled;
using Xunit;

/// <summary>
/// The subset of <c>profile-rest-api-v1</c> this session could check mechanically
/// WITHOUT an analyser — reflection over the compiled assembly.
/// </summary>
/// <remarks>
/// This class is the run's direct evidence for PRD §11.4. Per CG-R-127, every profile
/// rule is read-enforced for this run because no analyser exists; what follows is the
/// part that turned out not to need one, which is a smaller set than the profile's
/// <c>enforcement: analyser</c> markings claim.
///
/// The Gate C report states, rule by rule, which of the nine <c>must</c>/<c>must_not</c>
/// rules are checkable by reflection, which need a Roslyn syntax/semantic pass, and
/// which are not mechanically checkable at all under any reading. Two results there are
/// worth carrying: several rules are checkable only in their literal form, and
/// <c>EvadesEveryMustNot_ByIndirection</c> below PASSES — it asserts that this solution
/// performs persistence, transport reads and I/O while every role's <c>must_not</c>
/// holds. Q-16.
/// </remarks>
public sealed class ProfileConformanceTests
{
    private static readonly Assembly Slice = typeof(PlaceOrderHandler).Assembly;

    private static IEnumerable<Type> RoleTypes(SliceRole role) =>
        Slice.GetTypes()
            .Where(t => t.GetCustomAttribute<SliceAttribute>() is { } s
                        && s.ActInstance == "PlaceOrder"
                        && s.Role == role);

    /// <summary>controller must "declares [Slice(&lt;instance&gt;, \"controller\")]". Required: true.</summary>
    [Fact]
    public void A_controller_role_is_declared_for_PlaceOrder() =>
        Assert.Single(RoleTypes(SliceRole.Controller));

    /// <summary>handler must "declares [Slice(&lt;instance&gt;, \"handler\")]". Required: true.</summary>
    [Fact]
    public void A_handler_role_is_declared_for_PlaceOrder() =>
        Assert.Single(RoleTypes(SliceRole.Handler));

    /// <summary>
    /// provider — required: false, but required for THIS act: <c>ActorIdentity</c> is
    /// external per DSC-0003 and the vocabulary's own note.
    /// </summary>
    [Fact]
    public void Provider_roles_are_declared_for_the_external_read_position() =>
        Assert.Equal(2, RoleTypes(SliceRole.Provider).Count());

    /// <summary>
    /// handler must "exposes a single entry point taking the command and returning
    /// Accepted or Rejected". Checkable by reflection — this is one of the few that is.
    /// </summary>
    [Fact]
    public void The_handler_exposes_a_single_entry_point_over_the_command()
    {
        var entryPoints = typeof(PlaceOrderHandler)
            .GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Where(m => m.GetParameters().Length == 1
                        && m.GetParameters()[0].ParameterType == typeof(PlaceOrderCommand))
            .ToList();

        var entry = Assert.Single(entryPoints);
        Assert.Equal(typeof(PlaceOrderOutcome), entry.ReturnType);
    }

    /// <summary>
    /// handler must "emits only events the act declares it writes". <c>PlaceOrder</c>
    /// declares <c>writes: [OrderPlaced]</c>.
    /// </summary>
    /// <remarks>
    /// D-36 — this checks the SHAPE of the outcome type, not the emissions. A handler
    /// that constructed a second event type and dropped it would pass. Checking the
    /// real rule needs a Roslyn pass over object-creation expressions, and checking it
    /// against the act vocabulary needs the analyser to read
    /// <c>ordering.eventmodel.yaml</c> — which no input says any tool does.
    /// </remarks>
    [Fact]
    public void The_accepted_outcome_carries_only_the_declared_write()
    {
        var carried = typeof(PlaceOrderOutcome.Accepted)
            .GetProperties(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
            .Select(p => p.PropertyType.Name)
            .ToList();

        Assert.Equal(new[] { nameof(Ordering.Api.Facts.OrderPlaced) }, carried);
    }

    /// <summary>
    /// controller must_not "references a provider role directly" — checkable by
    /// reflection over the constructor signature only. A method-body reference or a
    /// service-locator call would not be caught here.
    /// </summary>
    [Fact]
    public void The_controller_does_not_take_a_provider_role()
    {
        var dependencies = typeof(PlaceOrderController)
            .GetConstructors()
            .SelectMany(c => c.GetParameters())
            .Select(p => p.ParameterType);

        Assert.DoesNotContain(dependencies, t => t.GetCustomAttribute<SliceAttribute>()?.Role == SliceRole.Provider);
    }

    /// <summary>
    /// THE FINDING. Every <c>must_not</c> in the profile holds while the slice does the
    /// forbidden work through unroled types. This test passing is the defect report.
    /// </summary>
    [Fact]
    public void EvadesEveryMustNot_ByIndirection()
    {
        // The unroled types that do the forbidden work carry no role at all,
        // so not one profile rule reaches them.
        foreach (var evader in new[]
                 {
                     typeof(EventAppendingPlaceOrderHandler),  // performs the act's I/O
                     typeof(HttpContextClaimSource),           // references a transport type
                     typeof(InMemoryOrderPlacedSink),          // is the persistence type
                 })
        {
            Assert.Null(evader.GetCustomAttribute<SliceAttribute>());
        }

        // And the controller's declared dependency is an interface, so its source text
        // reaches "exactly one type declaring the handler role" while the composition
        // root puts an unroled type first in the call chain.
        var dependency = Assert.Single(typeof(PlaceOrderController).GetConstructors()).GetParameters();
        Assert.Equal(typeof(IPlaceOrderHandler), Assert.Single(dependency).ParameterType);
        Assert.Null(typeof(IPlaceOrderHandler).GetCustomAttribute<SliceAttribute>());
    }
}
