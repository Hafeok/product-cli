// The reference graph, as edges from members to what their signatures and
// bodies name. Resolved through the semantic model — never by matching text.
// Each edge records HOW the target is used (schema version 3), because the
// consumer's denominator criterion is per edge:
//
//   call              member → method it invokes (or constructor it runs)
//   construct         member → type it instantiates
//   access            member → property, field or event it reads or writes
//   parameter         member → the declared type of one of its parameters
//   signature         member → its return type, or a field's/property's type
//   generic-argument  member → a type argument inside any of the above
//   type-test         member → a type it tests for or casts to (is/as/pattern/cast)
//   resolve           member → the type argument of a service-locator call
//                     (GetService/GetRequiredService and kin on IServiceProvider)
//   type-reference    member → any other named type its body mentions
//
// Targets outside the solution are kept when they are abstractions (an
// interface or an abstract class — what a container satisfies), for
// constructor parameters and service-locator arguments (composition facts
// whatever the type), and for `inherit`/`implement`/`attribute`; other
// external targets are dropped, or System.* would dominate the artefact.

using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace CSharpInventory;

public sealed class References
{
    private const string ServiceCollection = "T:Microsoft.Extensions.DependencyInjection.IServiceCollection";

    private const string ServiceProvider = "T:System.IServiceProvider";

    private readonly Compilation _compilation;
    private readonly Action<string, string, string> _add;
    private readonly Action<Registration> _register;
    private readonly Action<INamedTypeSymbol> _external;
    private readonly string _solutionDir;
    private readonly List<ISymbol> _pending = new();

    public References(Compilation compilation, Action<string, string, string> add, Action<Registration> register, Action<INamedTypeSymbol> external, string solutionDir)
    {
        _compilation = compilation;
        _add = add;
        _register = register;
        _external = external;
        _solutionDir = solutionDir;
    }

    public void Queue(ISymbol member) => _pending.Add(member);

    public void Flush()
    {
        foreach (var member in _pending) Walk(member);
        _pending.Clear();
    }

    private void Walk(ISymbol member)
    {
        var from = Ids.Of(member);
        foreach (var p in Ids.Parameters(member)) TypeRef(from, p.Type, "parameter");
        switch (member)
        {
            case IMethodSymbol m when !m.ReturnsVoid: TypeRef(from, m.ReturnType, "signature"); break;
            case IPropertySymbol p: TypeRef(from, p.Type, "signature"); break;
            case IFieldSymbol f: TypeRef(from, f.Type, "signature"); break;
            case IEventSymbol e: TypeRef(from, e.Type, "signature"); break;
        }
        foreach (var syntaxRef in member.DeclaringSyntaxReferences)
        {
            var node = syntaxRef.GetSyntax();
            var model = _compilation.GetSemanticModel(node.SyntaxTree);
            foreach (var n in node.DescendantNodes()) Body(model, from, n);
        }
    }

    private void Body(SemanticModel model, string from, SyntaxNode n)
    {
        switch (n)
        {
            case InvocationExpressionSyntax inv:
                if (Resolve(model, inv) is IMethodSymbol callee)
                {
                    var declared = callee.ReducedFrom ?? callee;
                    Edge(from, declared.OriginalDefinition, "call");
                    foreach (var ta in callee.TypeArguments) TypeRef(from, ta, "generic-argument");
                    if (OnServiceProvider(model, inv, callee))
                        foreach (var ta in callee.TypeArguments) TypeRef(from, ta, "resolve");
                    if (OnServiceCollection(model, inv, callee)) _register(RegistrationOf(model, from, inv, callee, declared));
                }
                break;
            case BaseObjectCreationExpressionSyntax creation:
                if (model.GetTypeInfo(creation).Type is INamedTypeSymbol created) Edge(from, created.OriginalDefinition, "construct");
                if (Resolve(model, creation) is IMethodSymbol ctor) Edge(from, ctor.OriginalDefinition, "call");
                break;
            case BinaryExpressionSyntax { RawKind: (int)SyntaxKind.IsExpression or (int)SyntaxKind.AsExpression } bin:
                if (model.GetTypeInfo(bin.Right).Type is { } tested) TypeRef(from, tested, "type-test");
                break;
            case IsPatternExpressionSyntax or DeclarationPatternSyntax or TypePatternSyntax or CastExpressionSyntax:
                foreach (var t in n.ChildNodes().OfType<TypeSyntax>())
                    if (model.GetTypeInfo(t).Type is { } patternType) TypeRef(from, patternType, "type-test");
                break;
            case IdentifierNameSyntax or GenericNameSyntax:
                switch (Resolve(model, n))
                {
                    case INamedTypeSymbol t: TypeRef(from, t, "type-reference"); break;
                    case IPropertySymbol or IFieldSymbol or IEventSymbol: Edge(from, Resolve(model, n)!.OriginalDefinition, "access"); break;
                }
                break;
        }
    }

    // A call whose receiver is (or implements) IServiceProvider — a service
    // locator: GetService<T>, GetRequiredService<T>, GetServices<T>, keyed kin.
    private static bool OnServiceProvider(SemanticModel model, InvocationExpressionSyntax inv, IMethodSymbol callee)
    {
        ITypeSymbol? receiver = callee.ReducedFrom is not null ? callee.ReceiverType
            : callee.IsExtensionMethod ? callee.Parameters.FirstOrDefault()?.Type
            : inv.Expression is MemberAccessExpressionSyntax ma ? model.GetTypeInfo(ma.Expression).Type
            : callee.ContainingType;
        if (receiver is null || callee.TypeArguments.Length == 0) return false;
        return Ids.Of(receiver.OriginalDefinition) == ServiceProvider
            || receiver.AllInterfaces.Any(i => Ids.Of(i.OriginalDefinition) == ServiceProvider);
    }

    // A call whose receiver is an IServiceCollection (or a type implementing
    // it), whether as an extension method or an instance method. Identity of
    // the framework interface, not a name test.
    private static bool OnServiceCollection(SemanticModel model, InvocationExpressionSyntax inv, IMethodSymbol callee)
    {
        ITypeSymbol? receiver = callee.ReducedFrom is not null ? callee.ReceiverType
            : callee.IsExtensionMethod ? callee.Parameters.FirstOrDefault()?.Type
            : inv.Expression is MemberAccessExpressionSyntax ma ? model.GetTypeInfo(ma.Expression).Type
            : callee.ContainingType;
        if (receiver is null) return false;
        return Ids.Of(receiver.OriginalDefinition) == ServiceCollection
            || receiver.AllInterfaces.Any(i => Ids.Of(i.OriginalDefinition) == ServiceCollection);
    }

    private Registration RegistrationOf(SemanticModel model, string site, InvocationExpressionSyntax inv, IMethodSymbol callee, IMethodSymbol declared)
    {
        var args = inv.ArgumentList.DescendantNodes().ToList();
        var typeofs = args.OfType<TypeOfExpressionSyntax>()
            .Select(t => model.GetTypeInfo(t.Type).Type).OfType<ITypeSymbol>()
            .Select(t => Ids.Of(t.OriginalDefinition)).Distinct().ToList();
        var constructs = args.OfType<BaseObjectCreationExpressionSyntax>()
            .Select(c => model.GetTypeInfo(c).Type).OfType<INamedTypeSymbol>()
            .Select(t => Ids.Of(t.OriginalDefinition)).Distinct().ToList();
        var span = inv.GetLocation().GetLineSpan();
        return new Registration
        {
            Site = site,
            Method = Ids.Of(declared.OriginalDefinition),
            MethodName = callee.Name,
            TypeArguments = callee.TypeArguments.Select(t => Ids.Of(t.OriginalDefinition)).ToList(),
            TypeofArguments = typeofs,
            Constructs = constructs,
            HasLambda = args.Any(n => n is AnonymousFunctionExpressionSyntax),
            Conditional = inv.Ancestors().TakeWhile(a => a is not MemberDeclarationSyntax)
                .Any(a => a is IfStatementSyntax or ConditionalExpressionSyntax or SwitchStatementSyntax or SwitchExpressionSyntax),
            File = Collector.Relative(_solutionDir, span.Path),
            Line = span.StartLinePosition.Line + 1,
        };
    }

    private static ISymbol? Resolve(SemanticModel model, SyntaxNode node)
    {
        var info = model.GetSymbolInfo(node);
        return info.Symbol ?? (info.CandidateSymbols.Length == 1 ? info.CandidateSymbols[0] : null);
    }

    // A type reference reaches every named type inside the type expression:
    // IHandler<PlaceOrder> names both IHandler`1 (with the given kind) and
    // PlaceOrder (as a generic argument).
    private void TypeRef(string from, ITypeSymbol type, string kind)
    {
        switch (type)
        {
            case INamedTypeSymbol named:
                Edge(from, named.OriginalDefinition, kind);
                foreach (var arg in named.TypeArguments) TypeRef(from, arg, "generic-argument");
                break;
            case IArrayTypeSymbol array: TypeRef(from, array.ElementType, kind); break;
            case IPointerTypeSymbol pointer: TypeRef(from, pointer.PointedAtType, kind); break;
        }
    }

    private void Edge(string from, ISymbol to, string kind)
    {
        var id = Ids.Of(to);
        if (id == from) return;
        // An external abstraction, a constructor parameter or a service-locator
        // argument is kept whatever its declaring assembly; its shape is noted
        // so the consumer's role proxies can read it. Everything else outside
        // the solution is dropped in Collector.Finish.
        var type = to as INamedTypeSymbol ?? to.ContainingType;
        if (type is not null && !type.Locations.Any(l => l.IsInSource))
        {
            var abstraction = type.TypeKind == TypeKind.Interface || type.IsAbstract;
            if (abstraction || kind == "parameter" || kind == "resolve") { _external(type); _add(from, id, kind + "+external"); return; }
        }
        _add(from, id, kind);
    }
}
