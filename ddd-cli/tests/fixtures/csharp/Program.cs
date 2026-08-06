// Fixture: exercises the three M2 finding kinds against the .editorconfig
// beside it. The pragma below is the in-source suppression the compiler
// still logs into SARIF (suppressions: [{kind: inSource}]).

using System.IO;

var unused = Analyze();
System.Console.WriteLine(unused);

static string Analyze()
{
#pragma warning disable CA2000 // Dispose objects before losing scope
    var reader = new StringReader("governed?");
#pragma warning restore CA2000
    return reader.ReadToEnd();
}
