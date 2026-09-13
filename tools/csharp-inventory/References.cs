// The reference graph, as edges from members to what their signatures and
// bodies name. Resolved through the semantic model — never by matching text.
//
//   call            member → method it invokes (or constructor it runs)
//   construct       member → type it instantiates
//   access          member → property, field or event it reads or writes
//   type-reference  member → named type in its signature or body
//
// Targets outside the solution are dropped for these four kinds (the artefact
// would otherwise be dominated by System.*); `inherit`, `implement` and
// `attribute` edges keep external targets, since a framework base type or a
// framework attribute is exactly what a root convention may name.

using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace CSharpInventory;

public sealed class References
{
    private readonly Compilation _compilation;
    private readonly HashSet<string> _inSolution;
    private readonly Action<string, string, string> _add;
    private readonly List<ISymbol> _pending = new();

    public References(Compilation compilation, HashSet<string> inSolution, Action<string, string, string> add)
    {
        _compilation = compilation;
        _inSolution = inSolution;
        _add = add;
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
        foreach (var p in Ids.Parameters(member)) TypeRef(from, p.Type);
        switch (member)
        {
            case IMethodSymbol m when !m.ReturnsVoid: TypeRef(from, m.ReturnType); break;
            case IPropertySymbol p: TypeRef(from, p.Type); break;
            case IFieldSymbol f: TypeRef(from, f.Type); break;
            case IEventSymbol e: TypeRef(from, e.Type); break;
        }
        foreach (var syntaxRef in member.DeclaringSyntaxReferences)
        {
            var node = syntaxRef.GetSyntax();
            var model = _compilation.GetSemanticModel(node.SyntaxTree);
            foreach (var n in node.DescendantNodes())
            {
                switch (n)
                {
                    case InvocationExpressionSyntax inv:
                        if (Resolve(model, inv) is IMethodSymbol callee) Edge(from, callee.OriginalDefinition, "call");
                        break;
                    case BaseObjectCreationExpressionSyntax creation:
                        if (model.GetTypeInfo(creation).Type is INamedTypeSymbol created) Edge(from, created.OriginalDefinition, "construct");
                        if (Resolve(model, creation) is IMethodSymbol ctor) Edge(from, ctor.OriginalDefinition, "call");
                        break;
                    case IdentifierNameSyntax or GenericNameSyntax:
                        switch (Resolve(model, n))
                        {
                            case INamedTypeSymbol t: TypeRef(from, t); break;
                            case IPropertySymbol or IFieldSymbol or IEventSymbol: Edge(from, Resolve(model, n)!.OriginalDefinition, "access"); break;
                        }
                        break;
                }
            }
        }
    }

    private static ISymbol? Resolve(SemanticModel model, SyntaxNode node)
    {
        var info = model.GetSymbolInfo(node);
        return info.Symbol ?? (info.CandidateSymbols.Length == 1 ? info.CandidateSymbols[0] : null);
    }

    // A type reference reaches every named type inside the type expression:
    // IHandler<PlaceOrder> names both IHandler`1 and PlaceOrder.
    private void TypeRef(string from, ITypeSymbol type)
    {
        switch (type)
        {
            case INamedTypeSymbol named:
                Edge(from, named.OriginalDefinition, "type-reference");
                foreach (var arg in named.TypeArguments) TypeRef(from, arg);
                break;
            case IArrayTypeSymbol array: TypeRef(from, array.ElementType); break;
            case IPointerTypeSymbol pointer: TypeRef(from, pointer.PointedAtType); break;
        }
    }

    private void Edge(string from, ISymbol to, string kind)
    {
        var id = Ids.Of(to);
        if (id == from) return;
        // Deferred in-solution filtering happens in Collector.Finish; edges to
        // types of a project loaded later must not be dropped here.
        _add(from, id, kind);
    }
}
