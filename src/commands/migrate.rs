//! Migration from monolithic PRD or ADR documents.

use clap::Subcommand;
use product_lib::{config, graph::inference, migrate};
use std::path::PathBuf;

use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Parse a monolithic PRD into feature files
    FromPrd {
        /// Path to the source PRD markdown file
        source: String,
        /// Only show what would be created, don't write files
        #[arg(long)]
        validate: bool,
        /// Write files (default: dry-run)
        #[arg(long)]
        execute: bool,
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Review each artifact before writing
        #[arg(long)]
        interactive: bool,
    },
    /// Parse a monolithic ADR file into ADR + test files
    FromAdrs {
        /// Path to the source ADR markdown file
        source: String,
        /// Only show what would be created
        #[arg(long)]
        validate: bool,
        /// Write files
        #[arg(long)]
        execute: bool,
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Review each artifact before writing
        #[arg(long)]
        interactive: bool,
    },
    /// Infer transitive TC→Feature links from shared ADRs (ADR-027)
    LinkTests {
        /// Only show what would be linked (don't write)
        #[arg(long)]
        dry_run: bool,
        /// Scope to a specific ADR
        #[arg(long)]
        adr: Option<String>,
        /// Scope to a specific feature
        #[arg(long)]
        feature: Option<String>,
    },
    /// Upgrade front-matter schema to current version
    Schema {
        /// Show what would change without writing
        #[arg(long)]
        dry_run: bool,
    },
    /// Report what migration would produce without writing
    Validate,
}

pub(crate) fn handle_migrate(cmd: MigrateCommands) -> BoxResult {
    match cmd {
        MigrateCommands::FromPrd {
            source,
            validate,
            execute,
            overwrite,
            interactive,
        } => migrate_from_prd(&source, validate, execute, overwrite, interactive),
        MigrateCommands::FromAdrs {
            source,
            validate,
            execute,
            overwrite,
            interactive,
        } => migrate_from_adrs(&source, validate, execute, overwrite, interactive),
        MigrateCommands::LinkTests {
            dry_run,
            adr,
            feature,
        } => migrate_link_tests(dry_run, adr, feature),
        MigrateCommands::Schema { dry_run } => migrate_schema(dry_run),
        MigrateCommands::Validate => migrate_validate(),
    }
}

fn migrate_from_prd(
    source: &str,
    validate: bool,
    execute: bool,
    overwrite: bool,
    interactive: bool,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (cfg, root, _) = load_graph()?;
    let features_dir = cfg.resolve_path(&root, &cfg.paths.features);
    let plan =
        migrate::migrate_from_prd(&PathBuf::from(source), &features_dir, &cfg.prefixes.feature)?;
    plan.print_summary();

    if validate || (!execute && !interactive) {
        println!("Run with --execute to create these files.");
    } else {
        std::fs::create_dir_all(&features_dir)?;
        let adrs_dir = cfg.resolve_path(&root, &cfg.paths.adrs);
        let tests_dir = cfg.resolve_path(&root, &cfg.paths.tests);
        let (written, skipped) =
            migrate::execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, overwrite, interactive)?;
        println!("\n{} files written, {} skipped", written, skipped);
    }
    Ok(())
}

fn migrate_from_adrs(
    source: &str,
    validate: bool,
    execute: bool,
    overwrite: bool,
    interactive: bool,
) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (cfg, root, _) = load_graph()?;
    let adrs_dir = cfg.resolve_path(&root, &cfg.paths.adrs);
    let tests_dir = cfg.resolve_path(&root, &cfg.paths.tests);
    let plan = migrate::migrate_from_adrs(
        &PathBuf::from(source),
        &adrs_dir,
        &tests_dir,
        &cfg.prefixes.adr,
        &cfg.prefixes.test,
    )?;
    plan.print_summary();

    if validate || (!execute && !interactive) {
        println!("Run with --execute to create these files.");
    } else {
        let features_dir = cfg.resolve_path(&root, &cfg.paths.features);
        std::fs::create_dir_all(&adrs_dir)?;
        std::fs::create_dir_all(&tests_dir)?;
        let (written, skipped) =
            migrate::execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, overwrite, interactive)?;
        println!("\n{} files written, {} skipped", written, skipped);
    }
    Ok(())
}

fn migrate_schema(dry_run: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (cfg, root, _) = load_graph()?;
    let version: u32 = cfg.schema_version.parse().unwrap_or(0);
    if version >= config::CURRENT_SCHEMA_VERSION {
        println!("Schema is already at version {} (current)", version);
    } else {
        println!(
            "Migrating schema from {} to {}{}",
            version,
            config::CURRENT_SCHEMA_VERSION,
            if dry_run { " (dry-run)" } else { "" }
        );
        let (updated, unchanged) = product_lib::config_migrate::migrate_schema(&root, &cfg, dry_run)?;
        println!("{} files updated, {} unchanged", updated, unchanged);
    }
    Ok(())
}

fn migrate_link_tests(dry_run: bool, adr: Option<String>, feature: Option<String>) -> BoxResult {
    let _lock = if !dry_run {
        Some(acquire_write_lock()?)
    } else {
        None
    };
    let (_, _, graph) = load_graph()?;
    let opts = inference::InferenceOptions {
        skip_cross_cutting: true,
        adr_filter: adr,
        feature_filter: feature,
    };
    inference::run_inference(&graph, &opts, dry_run)?;
    Ok(())
}

fn migrate_validate() -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let result = graph.check();
    result.print_stderr();
    println!(
        "Validation: {} errors, {} warnings",
        result.errors.len(),
        result.warnings.len()
    );
    Ok(())
}
