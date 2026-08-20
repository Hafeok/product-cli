//! Mock LSP host binary — fixture-grade stdio server for hermetic tests.
//!
//! Flags: `--handshake roslyn` demands `solution/open`/`project/open`
//! before readiness; `--over-advertise type-hierarchy` advertises the
//! provider then answers `null`; `DDD_MOCK_READY_DELAY_MS` delays the
//! readiness signal to make the loading window observable.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let roslyn_handshake = args
        .windows(2)
        .any(|w| w[0] == "--handshake" && w[1] == "roslyn");
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
