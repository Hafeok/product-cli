//! Codebase onboarding: scan, triage, seed (ADR-027).

use clap::Subcommand;
use product_lib::{error::ProductError, onboard, parser};
use std::path::PathBuf;

use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum OnboardCommands {
    /// Scan a codebase for decision candidates
    Scan {
        /// Path to the source directory to scan
        source: String,
        /// Output file for candidates JSON
        #[arg(long, default_value = "candidates.json")]
        output: String,
        /// Maximum number of candidates to produce
        #[arg(long)]
        max_candidates: Option<usize>,
        /// Disable evidence validation
        #[arg(long)]
        no_validate: bool,
    },
    /// Triage decision candidates (confirm, reject, merge)
    Triage {
        /// Path to candidates.json from scan phase
        source: String,
        /// Interactive triage (reads actions from stdin)
        #[arg(long)]
        interactive: bool,
        /// Output file for triaged candidates
        #[arg(long, default_value = "triaged.json")]
        output: String,
    },
    /// Seed the knowledge graph from triaged candidates
    Seed {
        /// Path to triaged.json from triage phase
        source: String,
        /// Show what would be created without writing files
        #[arg(long)]
        dry_run: bool,
    },
}

pub(crate) fn handle_onboard(cmd: OnboardCommands) -> BoxResult {
    match cmd {
        OnboardCommands::Scan {
            source,
            output,
            max_candidates,
            no_validate,
        } => onboard_scan(&source, &output, max_candidates, no_validate),
        OnboardCommands::Triage {
            source,
            interactive,
            output,
        } => onboard_triage(&source, interactive, &output),
        OnboardCommands::Seed { source, dry_run } => onboard_seed(&source, dry_run),
    }
}

fn onboard_scan(
    source: &str,
    output: &str,
    max_candidates: Option<usize>,
    no_validate: bool,
) -> BoxResult {
    let source_path = PathBuf::from(source);
    let scan_result = onboard::scan(
        &source_path,
        max_candidates,
        !no_validate,
    )?;

    let json = serde_json::to_string_pretty(&scan_result)
        .map_err(|e| ProductError::IoError(format!("failed to serialize scan output: {}", e)))?;

    write_output_file(output, &json)?;

    println!(
        "Scan complete: {} candidates from {} files",
        scan_result.candidates.len(),
        scan_result.scan_metadata.files_scanned
    );
    println!("Output written to {}", output);

    for c in &scan_result.candidates {
        for w in &c.warnings {
            eprintln!("warning: {} ({}): {}", c.id, c.title, w);
        }
    }

    Ok(())
}

fn onboard_triage(source: &str, interactive: bool, output: &str) -> BoxResult {
    let scan_output = read_scan_output(source)?;

    let triage_output = if interactive {
        let stdin = std::io::stdin();
        let mut reader = std::io::BufReader::new(stdin.lock());
        onboard::triage_interactive(&scan_output, &mut reader)?
    } else {
        onboard::triage_batch_confirm(&scan_output)
    };

    let json = serde_json::to_string_pretty(&triage_output)
        .map_err(|e| ProductError::IoError(format!("failed to serialize triage output: {}", e)))?;

    write_output_file(output, &json)?;
    print_triage_summary(&triage_output, output);
    Ok(())
}

fn read_scan_output(source: &str) -> Result<onboard::ScanOutput, Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let content = std::fs::read_to_string(&source_path).map_err(|e| {
        ProductError::IoError(format!(
            "cannot read candidates file {}: {}",
            source_path.display(),
            e
        ))
    })?;
    let scan_output: onboard::ScanOutput = serde_json::from_str(&content)
        .map_err(|e| {
            ProductError::IoError(format!(
                "cannot parse candidates file: {}",
                e
            ))
        })?;
    Ok(scan_output)
}

fn print_triage_summary(triage_output: &onboard::TriageOutput, output: &str) {
    let confirmed = triage_output
        .candidates
        .iter()
        .filter(|c| c.triage_status == onboard::TriageStatus::Confirmed)
        .count();
    let rejected = triage_output
        .candidates
        .iter()
        .filter(|c| c.triage_status == onboard::TriageStatus::Rejected)
        .count();
    let merged = triage_output
        .candidates
        .iter()
        .filter(|c| c.triage_status == onboard::TriageStatus::Merged)
        .count();

    println!(
        "Triage complete: {} confirmed, {} rejected, {} merged",
        confirmed, rejected, merged
    );
    println!("Output written to {}", output);
}

fn onboard_seed(source: &str, dry_run: bool) -> BoxResult {
    let _lock = if !dry_run {
        Some(acquire_write_lock()?)
    } else {
        None
    };

    let (config, root, _graph) = load_graph()?;
    let triage_output = read_triage_output(source)?;
    let seed_result = plan_seed_from_graph(&config, &root, &triage_output)?;

    if dry_run {
        print_seed_dry_run(&seed_result);
    } else {
        let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
        let features_dir = config.resolve_path(&root, &config.paths.features);
        println!("Seeding knowledge graph...\n");
        onboard::execute_seed(&seed_result, &adrs_dir, &features_dir)?;
        println!(
            "\nSeed complete: {} ADRs, {} feature stubs created",
            seed_result.adrs.len(),
            seed_result.features.len()
        );
    }

    Ok(())
}

fn read_triage_output(source: &str) -> Result<onboard::TriageOutput, Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let content = std::fs::read_to_string(&source_path).map_err(|e| {
        ProductError::IoError(format!(
            "cannot read triaged file {}: {}",
            source_path.display(),
            e
        ))
    })?;
    let triage_output: onboard::TriageOutput = serde_json::from_str(&content)
        .map_err(|e| {
            ProductError::IoError(format!(
                "cannot parse triaged file: {}",
                e
            ))
        })?;
    Ok(triage_output)
}

fn plan_seed_from_graph(
    config: &product_lib::config::ProductConfig,
    root: &std::path::Path,
    triage_output: &onboard::TriageOutput,
) -> Result<onboard::SeedResult, Box<dyn std::error::Error>> {
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let features_dir = config.resolve_path(root, &config.paths.features);
    let tests_dir = config.resolve_path(root, &config.paths.tests);

    let loaded = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
    let (features_all, adrs_all) = (loaded.features, loaded.adrs);

    let existing_adrs: Vec<String> = adrs_all.iter().map(|a| a.front.id.clone()).collect();
    let existing_features: Vec<String> = features_all.iter().map(|f| f.front.id.clone()).collect();

    Ok(onboard::plan_seed(
        triage_output,
        &existing_adrs,
        &existing_features,
        &config.prefixes.adr,
        &config.prefixes.feature,
    ))
}

fn print_seed_dry_run(seed_result: &onboard::SeedResult) {
    println!("Dry run \u{2014} the following files would be created:\n");
    println!("ADR files:");
    for adr in &seed_result.adrs {
        println!("  {} \u{2014} {} ({})", adr.id, adr.title, adr.filename);
    }
    println!("\nFeature stubs:");
    for ft in &seed_result.features {
        println!(
            "  {} \u{2014} {} ({}) -> [{}]",
            ft.id,
            ft.title,
            ft.filename,
            ft.adr_ids.join(", ")
        );
    }
    println!(
        "\nTotal: {} ADRs, {} feature stubs",
        seed_result.adrs.len(),
        seed_result.features.len()
    );
}

fn write_output_file(output: &str, json: &str) -> BoxResult {
    let output_path = PathBuf::from(output);
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ProductError::IoError(format!("cannot create output directory: {}", e))
            })?;
        }
    }
    std::fs::write(&output_path, json).map_err(|e| {
        ProductError::WriteError {
            path: output_path.clone(),
            message: e.to_string(),
        }
    })?;
    Ok(())
}
