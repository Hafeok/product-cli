//! The specification flow's MCP server.

#![deny(clippy::unwrap_used)]

fn main() {
    let root = std::env::args()
        .nth(1)
        .map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);
    if let Err(e) = spec_mcp::serve_stdio(root) {
        eprintln!("{e}");
        std::process::exit(2);
    }
}
