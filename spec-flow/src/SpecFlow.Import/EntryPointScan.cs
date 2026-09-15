using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace SpecFlow.Import;

/// <summary>Finds the places the outside world gets in.</summary>
/// <remarks>
/// Recognition is by attribute name, base-type name and invocation shape —
/// the conventions ASP.NET Core, the generic host and the common message
/// libraries actually use. A convention this misses shows up as a missing
/// entry point, never as a wrong one, which is the right way round: an
/// under-reporting importer loses candidates, an over-reporting one
/// manufactures acts nobody has.
/// </remarks>
public static class EntryPointScan
{
    private static readonly string[] HttpVerbAttributes =
        ["HttpGet", "HttpPost", "HttpPut", "HttpDelete", "HttpPatch", "HttpHead", "Route"];

    private static readonly string[] MinimalApiMaps =
        ["MapGet", "MapPost", "MapPut", "MapDelete", "MapPatch", "MapMethods"];

    private static readonly string[] HostedServiceBases =
        ["IHostedService", "BackgroundService", "IHostedLifecycleService"];

    private static readonly string[] HandlerInterfaces =
        ["IConsumer", "IRequestHandler", "INotificationHandler", "IMessageHandler", "IEventHandler"];

    /// <summary>Every entry point in one tree.</summary>
    public static List<EntryPoint> Scan(SyntaxTree tree, string relativePath)
    {
        var found = new List<EntryPoint>();
        var root = tree.GetRoot();

        foreach (var type in root.DescendantNodes().OfType<TypeDeclarationSyntax>())
        {
            var owner = SymbolScan.QualifiedName(type);
            found.AddRange(ControllerRoutes(type, owner, tree, relativePath));
            found.AddRange(ImplementedRoles(type, owner, tree, relativePath));
        }

        found.AddRange(MinimalApis(root, tree, relativePath));
        found.AddRange(ConsoleEntries(root, tree, relativePath));
        return found;
    }

    private static IEnumerable<EntryPoint> ControllerRoutes(
        TypeDeclarationSyntax type,
        string owner,
        SyntaxTree tree,
        string relativePath)
    {
        foreach (var method in type.Members.OfType<MethodDeclarationSyntax>())
        {
            foreach (var attribute in Attributes(method.AttributeLists))
            {
                var name = AttributeName(attribute);
                if (!HttpVerbAttributes.Contains(name, StringComparer.Ordinal))
                {
                    continue;
                }
                var template = FirstStringArgument(attribute) ?? method.Identifier.Text;
                yield return new EntryPoint(
                    $"{owner}.{method.Identifier.Text}#{name}",
                    "http-route",
                    $"{owner}.{method.Identifier.Text}",
                    $"{name} {template}",
                    relativePath,
                    SymbolScan.LineOf(tree, method));
            }
        }
    }

    private static IEnumerable<EntryPoint> ImplementedRoles(
        TypeDeclarationSyntax type,
        string owner,
        SyntaxTree tree,
        string relativePath)
    {
        foreach (var baseType in type.BaseList?.Types ?? default)
        {
            var name = BaseName(baseType.Type);
            if (HostedServiceBases.Contains(name, StringComparer.Ordinal))
            {
                yield return new EntryPoint(
                    $"{owner}#{name}", "hosted-service", owner, name,
                    relativePath, SymbolScan.LineOf(tree, type));
            }
            else if (HandlerInterfaces.Contains(name, StringComparer.Ordinal))
            {
                yield return new EntryPoint(
                    $"{owner}#{name}", "message-handler", owner,
                    $"{name}<{MessageArgument(baseType.Type)}>",
                    relativePath, SymbolScan.LineOf(tree, type));
            }
        }
    }

    private static IEnumerable<EntryPoint> MinimalApis(
        SyntaxNode root,
        SyntaxTree tree,
        string relativePath)
    {
        foreach (var invocation in root.DescendantNodes().OfType<InvocationExpressionSyntax>())
        {
            if (invocation.Expression is not MemberAccessExpressionSyntax member)
            {
                continue;
            }
            var name = member.Name.Identifier.Text;
            if (!MinimalApiMaps.Contains(name, StringComparer.Ordinal))
            {
                continue;
            }
            var route = invocation.ArgumentList.Arguments
                .Select(a => a.Expression)
                .OfType<LiteralExpressionSyntax>()
                .Select(l => l.Token.ValueText)
                .FirstOrDefault() ?? "(computed)";
            var line = SymbolScan.LineOf(tree, invocation);
            yield return new EntryPoint(
                $"{relativePath}:{line}#{name}", "minimal-api",
                $"{relativePath}:{line}", $"{name} {route}",
                relativePath, line);
        }
    }

    private static IEnumerable<EntryPoint> ConsoleEntries(
        SyntaxNode root,
        SyntaxTree tree,
        string relativePath)
    {
        foreach (var method in root.DescendantNodes().OfType<MethodDeclarationSyntax>())
        {
            if (method.Identifier.Text is not "Main"
                || !method.Modifiers.Any(m => m.ValueText is "static"))
            {
                continue;
            }
            var owner = method.Parent is BaseTypeDeclarationSyntax type
                ? SymbolScan.QualifiedName(type)
                : relativePath;
            yield return new EntryPoint(
                $"{owner}.Main#cli", "cli-entry", $"{owner}.Main", "Main",
                relativePath, SymbolScan.LineOf(tree, method));
        }
    }

    private static IEnumerable<AttributeSyntax> Attributes(SyntaxList<AttributeListSyntax> lists) =>
        lists.SelectMany(l => l.Attributes);

    private static string AttributeName(AttributeSyntax attribute)
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
            .FirstOrDefault();

    private static string BaseName(TypeSyntax type) => type switch
    {
        GenericNameSyntax generic => generic.Identifier.Text,
        QualifiedNameSyntax qualified => BaseName(qualified.Right),
        SimpleNameSyntax simple => simple.Identifier.Text,
        _ => type.ToString(),
    };

    private static string MessageArgument(TypeSyntax type) => type switch
    {
        GenericNameSyntax generic when generic.TypeArgumentList.Arguments.Count > 0
            => generic.TypeArgumentList.Arguments[0].ToString(),
        QualifiedNameSyntax qualified => MessageArgument(qualified.Right),
        _ => "?",
    };
}
