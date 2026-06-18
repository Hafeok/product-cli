//! Named-algorithm primitive validation + oracle conformance (§3.5).
//!
//! `product primitive {list,show,validate,check}` over `.product/primitives/`.
//! A primitive is authored (not derived — it has no Decider/Projector to simulate);
//! `validate` checks it declares a reference + I/O contract + oracle, and `check`
//! runs a realised implementation against the oracle pairs (§6.3).

use clap::Subcommand;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::primitive::{check_oracle, validate_primitive, Primitive};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum PrimitiveCommands {
    /// Check a realised implementation (a runner) against the oracle pairs (§6.3)
    Check {
        /// The primitive id (filename stem)
        name: String,
        /// Shell command that reads a JSON array of oracle inputs on stdin and
        /// writes a JSON array of outputs on stdout
        #[arg(long)]
        runner: String,
    },
    /// List the primitives under .product/primitives/
    List {},
    /// Show a primitive's declaration
    Show {
        /// The primitive id (filename stem)
        name: String,
    },
    /// Validate a primitive declares a reference + I/O contract + oracle
    Validate {
        /// The primitive id (filename stem)
        name: String,
    },
}

pub(crate) fn handle_primitive(cmd: PrimitiveCommands) -> BoxResult {
    match cmd {
        PrimitiveCommands::Check { name, runner } => check(&name, &runner),
        PrimitiveCommands::List {} => list(),
        PrimitiveCommands::Show { name } => show(&name),
        PrimitiveCommands::Validate { name } => validate(&name),
    }
}

fn primitives_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("primitives")
}

fn load(name: &str) -> Result<Primitive, Box<dyn std::error::Error>> {
    let p = primitives_dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&p)
        .map_err(|_| format!("no primitive '{name}' at {}", p.display()))?;
    Ok(Primitive::from_yaml(&text)?)
}

fn validate(name: &str) -> BoxResult {
    let prim = load(name)?;
    let results = validate_primitive(&prim);
    if has_blocking(&results) {
        eprintln!("non-conformant — {} violation(s):", results.len());
        for v in &results {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} primitive violation(s)", results.len()).into());
    }
    println!("conformant — primitive '{}' implements '{}' ({} oracle pair(s))", prim.id, prim.reference, prim.oracle.len());
    Ok(())
}

fn check(name: &str, runner: &str) -> BoxResult {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let prim = load(name)?;
    let inputs: Vec<&str> = prim.oracle.iter().map(|p| p.input.as_str()).collect();
    let input = serde_json::to_vec(&inputs)?;

    let mut child = Command::new("sh")
        .arg("-c").arg(runner)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start runner: {e}"))?;
    {
        let mut stdin = child.stdin.take().ok_or("runner has no stdin")?;
        stdin.write_all(&input)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("runner failed ({}): {}", output.status, String::from_utf8_lossy(&output.stderr)).into());
    }
    let realised: Vec<String> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("runner output is not a JSON array of strings: {e}"))?;
    let findings = check_oracle(&prim, &realised);
    if findings.is_empty() {
        println!("oracle-conformant — primitive '{}': {} pair(s) match the reference", prim.id, prim.oracle.len());
        return Ok(());
    }
    eprintln!("not conformant — {} finding(s):", findings.len());
    for f in &findings {
        eprintln!("  - [{}] {}: {}", f.focus, f.path, f.message);
    }
    Err(format!("{} oracle finding(s)", findings.len()).into())
}

fn show(name: &str) -> BoxResult {
    let p = load(name)?;
    println!("primitive: {}", p.id);
    println!("reference: {}", p.reference);
    println!("io: {} -> {}", p.input, p.output);
    println!("oracle: {} pair(s)", p.oracle.len());
    Ok(())
}

fn list() -> BoxResult {
    let dir = primitives_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        println!("(no primitives — author one under .product/primitives/)");
        return Ok(());
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no primitives)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
