// The inventory artefact, schema version 1. Mirrors
// schema/json/csharp-inventory/inventory.schema.json field for field; the
// schema is authoritative and the Rust side validates against it.
//
// Facts only. There is no field here that carries a judgement.

namespace CSharpInventory;

public sealed class Inventory
{
    public string InventoryVersion { get; init; } = "3";
    public ProducedBy ProducedBy { get; init; } = new();
    public string ProducedAt { get; init; } = "";
    public SolutionInfo Solution { get; init; } = new();
    public List<ProjectInfo> Projects { get; init; } = new();
    public List<TypeInfo> Types { get; init; } = new();
    public List<MemberInfo> Members { get; init; } = new();
    public List<Reference> References { get; init; } = new();
    public List<ExternalType> ExternalTypes { get; init; } = new();
    public List<Registration> Registrations { get; init; } = new();
    public List<Diagnostic> Diagnostics { get; init; } = new();
}

/// <summary>
/// A type declared outside the solution that an emitted edge lands on: its
/// kind and member shape, which the consumer's role proxies read. Facts only.
/// </summary>
public sealed class ExternalType
{
    public string Id { get; init; } = "";
    public string Assembly { get; init; } = "";
    public string Namespace { get; init; } = "";
    public string Name { get; init; } = "";
    public string Kind { get; init; } = "";
    public bool IsAbstract { get; init; }
    public int Arity { get; init; }
    public int Methods { get; init; }
    public int Properties { get; init; }
    public int Events { get; init; }
    /// <summary>Return types of the type's methods and properties that are themselves abstractions.</summary>
    public List<string> AbstractReturns { get; init; } = new();
}

/// <summary>
/// One call on an IServiceCollection, as written: the member it sits in, the
/// invoked method's identity, its generic and typeof arguments, what the
/// arguments construct, whether a lambda is among them, and whether the call
/// is inside a conditional. Every such call is recorded — AddControllers as
/// much as AddScoped — and which of them register what is decided Rust-side.
/// </summary>
public sealed class Registration
{
    public string Site { get; init; } = "";
    public string Method { get; init; } = "";
    public string MethodName { get; init; } = "";
    public List<string> TypeArguments { get; init; } = new();
    public List<string> TypeofArguments { get; init; } = new();
    public List<string> Constructs { get; init; } = new();
    public bool HasLambda { get; init; }
    public bool Conditional { get; init; }
    public string File { get; init; } = "";
    public int Line { get; init; }
}

public sealed class ProducedBy
{
    public string Tool { get; init; } = "csharp-inventory";
    public string ToolVersion { get; init; } = "0.1.0";
    public string RoslynVersion { get; init; } = "";
}

public sealed class SolutionInfo
{
    public string Path { get; init; } = "";
    public string? GitHead { get; init; }
}

public sealed class ProjectInfo
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Path { get; init; } = "";
    public List<string> TargetFrameworks { get; init; } = new();
    public string Assembly { get; init; } = "";
}

public sealed class AttributeUse
{
    public string Type { get; init; } = "";
    public List<object?> Positional { get; init; } = new();
    public Dictionary<string, object?> Named { get; init; } = new();
}

public sealed class TypeInfo
{
    public string Id { get; init; } = "";
    public string Project { get; init; } = "";
    public string Namespace { get; init; } = "";
    public string Name { get; init; } = "";
    public string Kind { get; init; } = "";
    public string Accessibility { get; init; } = "";
    public bool IsStatic { get; init; }
    public bool IsAbstract { get; init; }
    public bool IsPartial { get; init; }
    public string? BaseType { get; init; }
    public List<string> Interfaces { get; init; } = new();
    public string File { get; init; } = "";
    public int Line { get; init; }
    public List<AttributeUse> Attributes { get; init; } = new();
}

public sealed class ParameterInfo
{
    public string Name { get; init; } = "";
    public string Type { get; init; } = "";
}

public sealed class MemberInfo
{
    public string Id { get; init; } = "";
    public string DeclaringType { get; init; } = "";
    public string Name { get; init; } = "";
    public string Kind { get; init; } = "";
    public string Accessibility { get; init; } = "";
    public bool IsStatic { get; init; }
    public bool IsEntryPoint { get; init; }
    public List<ParameterInfo> Parameters { get; init; } = new();
    public string? ReturnType { get; init; }
    public string File { get; init; } = "";
    public int Line { get; init; }
    public List<AttributeUse> Attributes { get; init; } = new();
}

public sealed class Reference
{
    public string From { get; init; } = "";
    public string To { get; init; } = "";
    public string Kind { get; init; } = "";
}

public sealed class Diagnostic
{
    public string Severity { get; init; } = "";
    public string? Project { get; init; }
    public string Message { get; init; } = "";
}
