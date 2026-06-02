//! TC-889 fixture: a downstream consumer that depends only on product-core.
//!
//! Usage: `external-core-consumer <out-path>` — writes the sentinel string
//! `external-core ok` to <out-path> using `product_core::fileops::write_file_atomic`,
//! then prints `wrote: <path>` for the integration test to parse. The
//! integration test reads <out-path> back and asserts on its contents
//! (PAT-003 — assert causation, not just exit code).

use std::path::PathBuf;

fn main() {
    let out: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: external-core-consumer <out-path>");
    product_core::fileops::write_file_atomic(&out, "external-core ok")
        .expect("write sentinel");
    println!("wrote: {}", out.display());
}
