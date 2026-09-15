using System.Reflection;

namespace SpecFlow.Flow.Tests;

/// <summary>
/// The accountability boundary, asserted rather than asserted-in-prose.
/// </summary>
/// <remarks>
/// A rule held in a comment is not enforced. These tests fail if this
/// assembly ever grows a way to close an act-time record.
/// </remarks>
public class BoundaryTests
{
    [Fact]
    public void The_agent_host_refuses_to_assemble_a_close()
    {
        var thrown = Assert.Throws<InvalidOperationException>(
            () => SpecCli.Guard(["close", "01K5CJ7Q3S8XN2VYB4M6E9TZRA", "--nothing-arose"]));
        Assert.Contains("names a principal", thrown.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void The_forbidden_verb_is_refused_wherever_it_appears()
    {
        Assert.Throws<InvalidOperationException>(() => SpecCli.Guard(["--root", ".", "close"]));
    }

    [Fact]
    public void The_delegable_verbs_are_not_refused()
    {
        SpecCli.Guard(["implement", "--slice", "s", "--act", "act/a"]);
        SpecCli.Guard(["check", "--json"]);
        SpecCli.Guard(["records"]);
    }

    [Fact]
    public void No_public_member_of_the_flow_assembly_closes_a_record()
    {
        var offenders = typeof(SpecCli).Assembly
            .GetExportedTypes()
            .SelectMany(t => t.GetMembers(BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static))
            .Where(m => m.Name.Contains("Close", StringComparison.OrdinalIgnoreCase))
            .Select(m => $"{m.DeclaringType?.Name}.{m.Name}")
            .ToList();

        Assert.True(
            offenders.Count is 0,
            "the agent host must offer no way to close a record; found: " + string.Join(", ", offenders));
    }

    [Fact]
    public void The_hand_off_command_is_rendered_for_a_person_to_run()
    {
        var nothing = SpecCli.HandOffCommand("01ABC", []);
        Assert.Contains("--nothing-arose", nothing, StringComparison.Ordinal);
        Assert.Contains("--principal <you@example.com>", nothing, StringComparison.Ordinal);

        var filed = SpecCli.HandOffCommand("01ABC", ["det/a", "det/b"]);
        Assert.Contains("--determination det/a", filed, StringComparison.Ordinal);
        Assert.Contains("--determination det/b", filed, StringComparison.Ordinal);
        Assert.DoesNotContain("--nothing-arose", filed, StringComparison.Ordinal);
    }
}
