//! Mock LSP host binary for this crate's integration tests.
//!
//! Mirrors `ddd-lsp`'s `ddd-mock-lsp` (Cargo only exposes
//! `CARGO_BIN_EXE_*` for same-crate binaries, so each crate that tests
//! against the mock carries this thin wrapper).

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let roslyn_handshake = args.windows(2).any(|w| w[0] == "--handshake" && w[1] == "roslyn");
    let delay_ms = std::env::var("DDD_MOCK_READY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    let over_advertise_type_hierarchy = args
        .windows(2)
        .any(|w| w[0] == "--over-advertise" && w[1] == "type-hierarchy");
    ddd_lsp::mock::serve_stdio(ddd_lsp::mock::MockConfig {
        roslyn_handshake,
        ready_delay: std::time::Duration::from_millis(delay_ms),
        over_advertise_type_hierarchy,
    });
}
