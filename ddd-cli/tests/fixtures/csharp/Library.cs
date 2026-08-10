// Fixture: the M3/M4 contract-surface substrate. Every §9.1 policy row
// gets a triggering and a non-triggering edit case built on this file
// (ddd-lsp classify tests; the ddd-mcp interceptor walk).

public class PublicApi
{
    public PublicApi(int retries) { }
    public void Ping() { }
    public int Count { get; set; }
    protected void Guarded() { }
    private int hidden;
    private void Helper() { }
}

public interface IContract
{
    void Handle();
}

internal class Inner
{
    internal void Reach() { }
    public void LoudButCapped() { }
}

public class Tools
{
    [McpServerTool]
    private string Fetch() { return "x"; }
}

public enum Status
{
    Active,
    Suspended,
}

internal enum Toggle
{
    On,
    Off,
}
