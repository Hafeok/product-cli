using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace SpecFlow.Import;

/// <summary>Finds the claims code makes about the specification.</summary>
/// <remarks>
/// <para>
/// Two attributes, recognised <em>by name</em> rather than by type identity, so
/// a project declares its own one-line attribute classes and takes no package
/// dependency on this tool. The importer must run on codebases that have never
/// heard of it.
/// </para>
/// <para>
/// <c>[Slice("checkout-totals")]</c> claims this code is part of a slice built
/// against the specification. <c>[RealisesFact("det/…")]</c> claims it realises
/// a determination someone filed. Both are references, and a reference to
/// something that does not exist is broken — which is what makes them worth
/// scanning at all.
/// </para>
/// </remarks>
public static class AttributeScan
{
    /// <summary>The attribute claiming membership of a slice.</summary>
    public const string SliceAttribute = "Slice";

    /// <summary>The attribute claiming realisation of a filed determination.</summary>
    public const string RealisesFactAttribute = "RealisesFact";

    /// <summary>Every specification claim in one tree.</summary>
    public static List<SpecClaim> Scan(SyntaxTree tree, string relativePath)
    {
        var claims = new List<SpecClaim>();
        var root = tree.GetRoot();

        foreach (var node in root.DescendantNodes())
        {
            var (lists, symbol) = Target(node);
            if (lists is null || symbol is null)
            {
                continue;
            }
            foreach (var attribute in lists.Value.SelectMany(l => l.Attributes))
            {
                var kind = KindOf(attribute);
                if (kind is null)
                {
                    continue;
                }
                var value = FirstStringArgument(attribute);
                if (value is null)
                {
                    continue;
                }
                claims.Add(new SpecClaim(
                    kind, value, symbol, relativePath, SymbolScan.LineOf(tree, node)));
            }
        }

        return claims;
    }

    /// <summary>
    /// The attribute lists a node carries, and the symbol they attach to.
    /// </summary>
    /// <remarks>
    /// Types and methods only. An attribute on a parameter or a field is not
    /// refused, it is simply not a place this scan looks — under-reporting a
    /// claim loses a check, over-reporting one invents a finding.
    /// </remarks>
    private static (SyntaxList<AttributeListSyntax>? Lists, string? Symbol) Target(SyntaxNode node) =>
        node switch
        {
            BaseTypeDeclarationSyntax type =>
                (type.AttributeLists, SymbolScan.QualifiedName(type)),
            MethodDeclarationSyntax method when method.Parent is BaseTypeDeclarationSyntax owner =>
                (method.AttributeLists,
                 $"{SymbolScan.QualifiedName(owner)}.{method.Identifier.Text}"),
            _ => (null, null),
        };

    /// <summary>Which claim an attribute makes, or <c>null</c> for anything else.</summary>
    public static string? KindOf(AttributeSyntax attribute)
    {
        var name = BareName(attribute);
        return name switch
        {
            SliceAttribute => "slice",
            RealisesFactAttribute => "realises-fact",
            _ => null,
        };
    }

    private static string BareName(AttributeSyntax attribute)
    {
        var name = attribute.Name switch
        {
            QualifiedNameSyntax qualified => qualified.Right.Identifier.Text,
            SimpleNameSyntax simple => simple.Identifier.Text,
            _ => attribute.Name.ToString(),
        };
        return name.EndsWith("Attribute", StringComparison.Ordinal)
            ? name[..^"Attribute".Length]
            : name;
    }

    private static string? FirstStringArgument(AttributeSyntax attribute) =>
        attribute.ArgumentList?.Arguments
            .Select(a => a.Expression)
            .OfType<LiteralExpressionSyntax>()
            .Select(l => l.Token.ValueText)
            .FirstOrDefault(v => !string.IsNullOrWhiteSpace(v));
}
