//! Migration execution — write planned artifacts to disk (ADR-017)

use crate::error::Result;
use crate::types::*;
use std::path::Path;

use super::types::*;

/// Execute a migration plan: write files
/// If `interactive` is true, prompt for each artifact before writing.
pub fn execute_plan(
    plan: &MigrationPlan,
    features_dir: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
    overwrite: bool,
    interactive: bool,
) -> Result<(usize, usize)> {
    let mut written = 0;
    let mut skipped = 0;

    for f in &plan.features {
        let path = features_dir.join(&f.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", f.filename);
            continue;
        }
        if interactive {
            println!("\n--- Feature: {} — {} (phase {}) ---", f.id, f.title, f.phase);
            let preview = if f.body.len() > 200 { &f.body[..200] } else { &f.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = FeatureFrontMatter {
            id: f.id.clone(),
            title: f.title.clone(),
            phase: f.phase,
            status: f.status,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
            bundle: None,
        };
        let content = crate::parser::render_feature(&front, &f.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", f.filename);
    }

    for a in &plan.adrs {
        let path = adrs_dir.join(&a.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", a.filename);
            continue;
        }
        if interactive {
            println!("\n--- ADR: {} — {} ({}) ---", a.id, a.title, a.status);
            let preview = if a.body.len() > 200 { &a.body[..200] } else { &a.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = AdrFrontMatter {
            id: a.id.clone(),
            title: a.title.clone(),
            status: a.status,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: crate::types::AdrScope::Domain,
            content_hash: None,
            amendments: vec![],
            source_files: vec![],
        };
        let content = crate::parser::render_adr(&front, &a.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", a.filename);
    }

    for t in &plan.tests {
        let path = tests_dir.join(&t.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", t.filename);
            continue;
        }
        if interactive {
            println!("\n--- Test: {} — {} ({}, adr: {}) ---", t.id, t.title, t.test_type, t.adr_id);
            let preview = if t.body.len() > 200 { &t.body[..200] } else { &t.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = TestFrontMatter {
            id: t.id.clone(),
            title: t.title.clone(),
            test_type: t.test_type,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock {
                features: vec![],
                adrs: vec![t.adr_id.clone()],
            },
            phase: 1,
            content_hash: None,
            runner: None,
            runner_args: None,
            runner_timeout: None,
            requires: vec![],
            last_run: None,
            failure_message: None,
            last_run_duration: None,
        };
        let content = crate::parser::render_test(&front, &t.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", t.filename);
    }

    Ok((written, skipped))
}

// ---------------------------------------------------------------------------
// Interactive migration prompt (ADR-017)
// ---------------------------------------------------------------------------

pub(crate) enum InteractiveChoice {
    Accept,
    Skip,
    Quit,
}

pub(crate) fn prompt_interactive() -> Result<InteractiveChoice> {
    use std::io::{self, BufRead, Write};

    loop {
        print!("[a]ccept / [s]kip / [q]uit: ");
        io::stdout().flush().map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;

        match input.trim().to_lowercase().as_str() {
            "a" | "accept" => return Ok(InteractiveChoice::Accept),
            "s" | "skip" => return Ok(InteractiveChoice::Skip),
            "q" | "quit" => return Ok(InteractiveChoice::Quit),
            _ => println!("  Invalid choice. Enter a, s, or q."),
        }
    }
}
