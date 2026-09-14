// Symbol identity and the closed vocabularies the artefact uses for kinds
// and accessibility. Pure functions over Roslyn symbols.

using Microsoft.CodeAnalysis;

namespace CSharpInventory;

public static class Ids
{
    /// <summary>
    /// Documentation-comment ids of top-level types declared by more than
    /// one project in the solution (the top-level-statements `Program` of
    /// every web project, for one). Their ids, and their members', carry an
    /// `@assembly` suffix so two declarations never collapse into one.
    /// Filled once by <see cref="Collector.FindAmbiguous"/> before collection.
    /// </summary>
    public static HashSet<string> Ambiguous { get; } = new(StringComparer.Ordinal);

    // Documentation-comment ids cover every named symbol. Arrays and pointers
    // have none of their own and are rendered from their element type; a
    // generic type parameter keeps Roslyn's "!:" prefix — it names no type.
    public static string Of(ISymbol symbol)
    {
        var id = Bare(symbol);
        var outer = Outermost(symbol);
        if (outer is null || !Ambiguous.Contains(Bare(outer))) return id;
        return id + "@" + (outer.ContainingAssembly?.Name ?? "?");
    }

    /// <summary>The id without any disambiguating suffix.</summary>
    public static string Bare(ISymbol symbol) => symbol switch
    {
        IArrayTypeSymbol array => Of(array.ElementType) + "[]",
        IPointerTypeSymbol pointer => Of(pointer.PointedAtType) + "*",
        ITypeParameterSymbol tp => "!:" + tp.Name,
        _ => symbol.GetDocumentationCommentId() ?? "!:" + symbol.ToDisplayString(),
    };

    private static INamedTypeSymbol? Outermost(ISymbol symbol)
    {
        var type = symbol as INamedTypeSymbol ?? symbol.ContainingType;
        while (type?.ContainingType is not null) type = type.ContainingType;
        return type;
    }

    // The `Program` type that top-level statements generate is implicitly
    // declared yet carries the entry point, so it is kept; compiler closures
    // and display classes (names beginning with '<') are not.
    public static IEnumerable<INamedTypeSymbol> SourceTypes(INamespaceSymbol ns)
    {
        foreach (var type in ns.GetTypeMembers())
            foreach (var t in WithNested(type))
                if (!t.Name.StartsWith('<') && t.Locations.Any(l => l.IsInSource))
                    yield return t;
        foreach (var child in ns.GetNamespaceMembers())
            foreach (var t in SourceTypes(child))
                yield return t;
    }

    private static IEnumerable<INamedTypeSymbol> WithNested(INamedTypeSymbol type)
    {
        yield return type;
        foreach (var nested in type.GetTypeMembers())
            foreach (var t in WithNested(nested))
                yield return t;
    }

    public static bool IsRecordedMember(ISymbol member)
    {
        if (member.IsImplicitlyDeclared) return false;
        return member switch
        {
            IMethodSymbol m => m.MethodKind is MethodKind.Ordinary or MethodKind.Constructor
                or MethodKind.StaticConstructor or MethodKind.UserDefinedOperator or MethodKind.Conversion
                or MethodKind.Destructor,
            IPropertySymbol or IFieldSymbol or IEventSymbol => true,
            _ => false,
        };
    }

    public static string TypeKind(INamedTypeSymbol type) => type.TypeKind switch
    {
        Microsoft.CodeAnalysis.TypeKind.Class => type.IsRecord ? "record" : "class",
        Microsoft.CodeAnalysis.TypeKind.Struct => type.IsRecord ? "record-struct" : "struct",
        Microsoft.CodeAnalysis.TypeKind.Interface => "interface",
        Microsoft.CodeAnalysis.TypeKind.Enum => "enum",
        Microsoft.CodeAnalysis.TypeKind.Delegate => "delegate",
        _ => type.TypeKind.ToString().ToLowerInvariant(),
    };

    public static string MemberKind(ISymbol member) => member switch
    {
        IMethodSymbol { MethodKind: MethodKind.Constructor or MethodKind.StaticConstructor } => "constructor",
        IMethodSymbol => "method",
        IPropertySymbol => "property",
        IFieldSymbol => "field",
        IEventSymbol => "event",
        _ => "other",
    };

    public static string Accessibility(Microsoft.CodeAnalysis.Accessibility a) => a switch
    {
        Microsoft.CodeAnalysis.Accessibility.Public => "public",
        Microsoft.CodeAnalysis.Accessibility.Internal => "internal",
        Microsoft.CodeAnalysis.Accessibility.Protected => "protected",
        Microsoft.CodeAnalysis.Accessibility.Private => "private",
        Microsoft.CodeAnalysis.Accessibility.ProtectedOrInternal => "protected-internal",
        Microsoft.CodeAnalysis.Accessibility.ProtectedAndInternal => "private-protected",
        _ => "not-applicable",
    };

    public static IEnumerable<IParameterSymbol> Parameters(ISymbol member) => member switch
    {
        IMethodSymbol m => m.Parameters,
        IPropertySymbol p => p.Parameters,
        _ => Array.Empty<IParameterSymbol>(),
    };

    public static string? ReturnType(ISymbol member) => member switch
    {
        IMethodSymbol m => m.ReturnsVoid ? null : Of(m.ReturnType.OriginalDefinition),
        IPropertySymbol p => Of(p.Type.OriginalDefinition),
        IFieldSymbol f => Of(f.Type.OriginalDefinition),
        IEventSymbol e => Of(e.Type.OriginalDefinition),
        _ => null,
    };

    // A typed constant rendered as a JSON scalar. `typeof(X)` becomes the
    // string "T:X" — the one lossy case, stated in the Gate 0 schema proposal.
    // An erroneous constant (an attribute argument the compilation could not
    // bind) has no value; it is recorded as null rather than aborting the run.
    public static object? Constant(TypedConstant c)
    {
        try
        {
            return c.Kind switch
            {
                TypedConstantKind.Primitive => c.Value,
                TypedConstantKind.Enum => c.Value,
                TypedConstantKind.Type => c.Value is ITypeSymbol t ? Of(t) : null,
                TypedConstantKind.Array when !c.Values.IsDefault => c.Values.Select(Constant).ToList(),
                _ => null,
            };
        }
        catch (Exception)
        {
            return null;
        }
    }
}
