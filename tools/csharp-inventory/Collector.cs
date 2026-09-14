// Walks each project's compilation and records types, members and their
// declared attributes. Symbol identity is Roslyn's documentation-comment id
// (T:/M:/P:/F:/E:), which is exact over overloads and generic arity.
//
// What is recorded and what is not is a scope rule, stated once here and in
// the schema: every in-source named type of every project, every explicit
// member of those types, every attribute on them with every argument. Nothing
// is filtered by name, namespace, base type or attribute identity.

using Microsoft.CodeAnalysis;

namespace CSharpInventory;

public sealed class Collector
{
    private readonly string _solutionDir;
    private readonly List<Diagnostic> _diagnostics;
    private readonly List<ProjectInfo> _projects = new();
    private readonly Dictionary<string, TypeInfo> _types = new(StringComparer.Ordinal);
    private readonly Dictionary<string, MemberInfo> _members = new(StringComparer.Ordinal);
    private readonly HashSet<(string, string, string)> _refs = new();
    private readonly HashSet<string> _inSolution = new(StringComparer.Ordinal);
    private readonly List<Registration> _registrations = new();
    private string _roslyn = "";

    public Collector(string solutionDir, List<Diagnostic> diagnostics)
    {
        _solutionDir = solutionDir;
        _diagnostics = diagnostics;
    }

    /// <summary>
    /// First pass: which top-level type ids are declared by more than one
    /// project. Must run over every project before any is collected.
    /// </summary>
    public static async Task FindAmbiguous(IEnumerable<Project> projects)
    {
        var owners = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
        foreach (var project in projects)
        {
            var compilation = await project.GetCompilationAsync();
            if (compilation is null) continue;
            foreach (var type in Ids.SourceTypes(compilation.Assembly.GlobalNamespace))
            {
                if (type.ContainingType is not null) continue;
                var id = Ids.Bare(type);
                if (!owners.TryGetValue(id, out var set)) owners[id] = set = new HashSet<string>(StringComparer.Ordinal);
                set.Add(compilation.Assembly.Name);
            }
        }
        foreach (var (id, set) in owners)
            if (set.Count > 1) Ids.Ambiguous.Add(id);
    }

    public async Task AddProject(Project project)
    {
        var compilation = await project.GetCompilationAsync();
        if (compilation is null)
        {
            _diagnostics.Add(new Diagnostic { Severity = "error", Project = "P:" + project.Name, Message = "no compilation" });
            return;
        }
        _roslyn = typeof(Compilation).Assembly.GetName().Version?.ToString() ?? "";
        var pid = "P:" + project.Name;
        _projects.Add(new ProjectInfo
        {
            Id = pid,
            Name = project.Name,
            Path = Relative(_solutionDir, project.FilePath ?? ""),
            TargetFrameworks = TargetFrameworks(project),
            Assembly = project.AssemblyName,
        });
        // Compile errors are facts about load quality: a project that did not
        // resolve its references yields a semantic model full of error types,
        // and the consumer must be able to see that rather than infer it.
        var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToList();
        if (errors.Count > 0)
        {
            _diagnostics.Add(new Diagnostic
            {
                Severity = "warning",
                Project = pid,
                Message = $"{errors.Count} compile error(s); first: {errors[0].Id} {errors[0].GetMessage()}",
            });
        }
        var entryPoint = compilation.GetEntryPoint(CancellationToken.None);
        var entryId = entryPoint is null ? null : Ids.Of(entryPoint);
        var references = new References(compilation, AddRef, r => _registrations.Add(r), _solutionDir);

        foreach (var type in Ids.SourceTypes(compilation.Assembly.GlobalNamespace))
        {
            var tid = Ids.Of(type);
            _inSolution.Add(tid);
            if (_types.ContainsKey(tid)) continue;
            var (file, line) = Location(type);
            _types[tid] = new TypeInfo
            {
                Id = tid,
                Project = pid,
                Namespace = type.ContainingNamespace.IsGlobalNamespace ? "" : type.ContainingNamespace.ToDisplayString(),
                Name = type.Name,
                Kind = Ids.TypeKind(type),
                Accessibility = Ids.Accessibility(type.DeclaredAccessibility),
                IsStatic = type.IsStatic,
                IsAbstract = type.IsAbstract && type.TypeKind != TypeKind.Interface,
                IsPartial = type.DeclaringSyntaxReferences.Length > 1,
                BaseType = type.BaseType is null ? null : Ids.Of(type.BaseType.OriginalDefinition),
                Interfaces = type.Interfaces.Select(i => Ids.Of(i.OriginalDefinition)).OrderBy(s => s, StringComparer.Ordinal).ToList(),
                File = file,
                Line = line,
                Attributes = Attributes(type),
            };
            if (type.BaseType is not null) AddRef(tid, Ids.Of(type.BaseType.OriginalDefinition), "inherit");
            foreach (var i in type.Interfaces) AddRef(tid, Ids.Of(i.OriginalDefinition), "implement");
            foreach (var a in type.GetAttributes()) if (a.AttributeClass is not null) AddRef(tid, Ids.Of(a.AttributeClass), "attribute");

            foreach (var member in type.GetMembers())
            {
                var isEntry = entryPoint is not null && SymbolEqualityComparer.Default.Equals(member, entryPoint);
                if (!Ids.IsRecordedMember(member) && !isEntry) continue;
                var mid = Ids.Of(member);
                _inSolution.Add(mid);
                if (_members.ContainsKey(mid)) continue;
                var (mfile, mline) = Location(member);
                _members[mid] = new MemberInfo
                {
                    Id = mid,
                    DeclaringType = tid,
                    Name = member.Name,
                    Kind = Ids.MemberKind(member),
                    Accessibility = Ids.Accessibility(member.DeclaredAccessibility),
                    IsStatic = member.IsStatic,
                    IsEntryPoint = mid == entryId,
                    Parameters = Ids.Parameters(member).Select(p => new ParameterInfo { Name = p.Name, Type = Ids.Of(p.Type.OriginalDefinition) }).ToList(),
                    ReturnType = Ids.ReturnType(member),
                    File = mfile,
                    Line = mline,
                    Attributes = Attributes(member),
                };
                foreach (var a in member.GetAttributes()) if (a.AttributeClass is not null) AddRef(mid, Ids.Of(a.AttributeClass), "attribute");
                references.Queue(member);
            }
        }
        // Reference edges are resolved after every type of every project is
        // known, so in-solution filtering sees the whole solution.
        references.Flush();
    }

    public Inventory Finish(SolutionInfo solution)
    {
        return new Inventory
        {
            ProducedBy = new ProducedBy { RoslynVersion = _roslyn },
            ProducedAt = DateTime.UtcNow.ToString("o"),
            Solution = solution,
            Projects = _projects.OrderBy(p => p.Id, StringComparer.Ordinal).ToList(),
            Types = _types.Values.OrderBy(t => t.Id, StringComparer.Ordinal).ToList(),
            Members = _members.Values.OrderBy(m => m.Id, StringComparer.Ordinal).ToList(),
            References = _refs
                .Where(r => r.Item3 is "inherit" or "implement" or "attribute" || _inSolution.Contains(r.Item2))
                .OrderBy(r => r.Item1, StringComparer.Ordinal).ThenBy(r => r.Item2, StringComparer.Ordinal).ThenBy(r => r.Item3, StringComparer.Ordinal)
                .Select(r => new Reference { From = r.Item1, To = r.Item2, Kind = r.Item3 }).ToList(),
            Registrations = _registrations.OrderBy(r => r.Site, StringComparer.Ordinal).ThenBy(r => r.Line).ThenBy(r => r.Method, StringComparer.Ordinal).ToList(),
            Diagnostics = _diagnostics.OrderBy(d => d.Message, StringComparer.Ordinal).ToList(),
        };
    }

    private void AddRef(string from, string to, string kind) => _refs.Add((from, to, kind));

    private static List<string> TargetFrameworks(Project project)
    {
        // The parsed compilation carries no TFM; the project's options do not
        // expose it either. Recorded from the workspace's project name suffix
        // when multi-targeted (Name(net8.0)), else empty — an honest gap.
        var open = project.Name.LastIndexOf('(');
        if (open > 0 && project.Name.EndsWith(')'))
            return new List<string> { project.Name[(open + 1)..^1] };
        return new List<string>();
    }

    private (string, int) Location(ISymbol symbol)
    {
        var loc = symbol.Locations.FirstOrDefault(l => l.IsInSource);
        if (loc is null) return ("", 0);
        var span = loc.GetLineSpan();
        return (Relative(_solutionDir, span.Path), span.StartLinePosition.Line + 1);
    }

    private static List<AttributeUse> Attributes(ISymbol symbol) =>
        symbol.GetAttributes()
            .Where(a => a.AttributeClass is not null)
            .Select(a => new AttributeUse
            {
                Type = Ids.Of(a.AttributeClass!),
                Positional = a.ConstructorArguments.Select(Ids.Constant).ToList(),
                Named = a.NamedArguments.ToDictionary(kv => kv.Key, kv => Ids.Constant(kv.Value), StringComparer.Ordinal),
            })
            .ToList();

    public static string Relative(string root, string path)
    {
        if (string.IsNullOrEmpty(path)) return "";
        return Path.GetRelativePath(root, path).Replace('\\', '/');
    }
}
