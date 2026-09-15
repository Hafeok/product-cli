using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace SpecFlow.Import;

/// <summary>Walks a parsed tree for declarations, plus what they compose.</summary>
/// <remarks>
/// Syntax-first by design. The scan runs with whatever references happen to
/// resolve and never requires a restored, buildable project — an importer that
/// only works on a green build is an importer that does not run on the
/// codebases most worth importing.
/// </remarks>
public static class SymbolScan
{
    /// <summary>Declarations and composition edges in one tree.</summary>
    public static (List<SymbolEntry> Symbols, List<CompositionEdge> Edges) Scan(
        SyntaxTree tree,
        string relativePath)
    {
        var symbols = new List<SymbolEntry>();
        var edges = new List<CompositionEdge>();
        var root = tree.GetRoot();

        foreach (var declaration in root.DescendantNodes().OfType<BaseTypeDeclarationSyntax>())
        {
            var id = QualifiedName(declaration);
            symbols.Add(new SymbolEntry(
                id,
                KindOf(declaration),
                declaration.Identifier.Text,
                NamespaceOf(declaration),
                relativePath,
                LineOf(tree, declaration)));

            edges.AddRange(EdgesFrom(declaration, id));
            symbols.AddRange(MembersOf(declaration, id, tree, relativePath));
        }

        return (symbols, edges);
    }

    private static IEnumerable<SymbolEntry> MembersOf(
        BaseTypeDeclarationSyntax declaration,
        string owner,
        SyntaxTree tree,
        string relativePath)
    {
        if (declaration is not TypeDeclarationSyntax type)
        {
            yield break;
        }

        foreach (var method in type.Members.OfType<MethodDeclarationSyntax>())
        {
            yield return new SymbolEntry(
                $"{owner}.{method.Identifier.Text}",
                "method",
                method.Identifier.Text,
                NamespaceOf(declaration),
                relativePath,
                LineOf(tree, method));
        }
    }

    private static IEnumerable<CompositionEdge> EdgesFrom(BaseTypeDeclarationSyntax declaration, string from)
    {
        foreach (var baseType in declaration.BaseList?.Types ?? default)
        {
            yield return new CompositionEdge(from, TypeName(baseType.Type), "base");
        }

        if (declaration is not TypeDeclarationSyntax type)
        {
            yield break;
        }

        foreach (var field in type.Members.OfType<FieldDeclarationSyntax>())
        {
            yield return new CompositionEdge(from, TypeName(field.Declaration.Type), "field");
        }
        foreach (var property in type.Members.OfType<PropertyDeclarationSyntax>())
        {
            yield return new CompositionEdge(from, TypeName(property.Type), "property");
        }
        foreach (var parameter in type.Members
                     .OfType<ConstructorDeclarationSyntax>()
                     .SelectMany(c => c.ParameterList.Parameters))
        {
            if (parameter.Type is not null)
            {
                yield return new CompositionEdge(from, TypeName(parameter.Type), "ctor-param");
            }
        }
        foreach (var parameter in type.ParameterList?.Parameters ?? default)
        {
            if (parameter.Type is not null)
            {
                yield return new CompositionEdge(from, TypeName(parameter.Type), "primary-ctor-param");
            }
        }
    }

    /// <summary>The bare type name, unwrapped from nullability and generics.</summary>
    public static string TypeName(TypeSyntax type) => type switch
    {
        NullableTypeSyntax nullable => TypeName(nullable.ElementType),
        ArrayTypeSyntax array => TypeName(array.ElementType),
        GenericNameSyntax generic when generic.TypeArgumentList.Arguments.Count is 1
            => TypeName(generic.TypeArgumentList.Arguments[0]),
        GenericNameSyntax generic => generic.Identifier.Text,
        QualifiedNameSyntax qualified => TypeName(qualified.Right),
        _ => type.ToString(),
    };

    /// <summary>Namespace-qualified, nested types joined by dots.</summary>
    public static string QualifiedName(BaseTypeDeclarationSyntax declaration)
    {
        var parts = new List<string> { declaration.Identifier.Text };
        for (var parent = declaration.Parent; parent is not null; parent = parent.Parent)
        {
            if (parent is BaseTypeDeclarationSyntax outer)
            {
                parts.Insert(0, outer.Identifier.Text);
            }
        }
        var ns = NamespaceOf(declaration);
        var name = string.Join('.', parts);
        return ns.Length is 0 ? name : $"{ns}.{name}";
    }

    /// <summary>The declaration's namespace, or empty at global scope.</summary>
    public static string NamespaceOf(SyntaxNode node)
    {
        for (var parent = node.Parent; parent is not null; parent = parent.Parent)
        {
            if (parent is BaseNamespaceDeclarationSyntax ns)
            {
                return ns.Name.ToString();
            }
        }
        return string.Empty;
    }

    private static string KindOf(BaseTypeDeclarationSyntax declaration) => declaration switch
    {
        RecordDeclarationSyntax => "record",
        ClassDeclarationSyntax => "class",
        InterfaceDeclarationSyntax => "interface",
        StructDeclarationSyntax => "struct",
        EnumDeclarationSyntax => "enum",
        _ => "type",
    };

    /// <summary>One-based line of a node's first token.</summary>
    public static int LineOf(SyntaxTree tree, SyntaxNode node) =>
        tree.GetLineSpan(node.Span).StartLinePosition.Line + 1;
}
