//! Migration data types (ADR-017)

use crate::types::*;

#[derive(Debug)]
pub struct MigrationPlan {
    pub features: Vec<ProposedFeature>,
    pub adrs: Vec<ProposedAdr>,
    pub tests: Vec<ProposedTest>,
    pub warnings: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug)]
pub struct ProposedFeature {
    pub id: String,
    pub title: String,
    pub phase: u32,
    pub status: FeatureStatus,
    pub body: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct ProposedAdr {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub body: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct ProposedTest {
    pub id: String,
    pub title: String,
    pub test_type: TestType,
    pub adr_id: String,
    pub body: String,
    pub filename: String,
}

impl MigrationPlan {
    pub fn print_summary(&self) {
        println!(
            "Migration plan: {} features, {} ADRs, {} test criteria",
            self.features.len(),
            self.adrs.len(),
            self.tests.len()
        );
        println!();

        if !self.features.is_empty() {
            println!("Feature files to create:");
            for f in &self.features {
                println!("  {} (phase: {}, status: {})", f.filename, f.phase, f.status);
            }
            println!();
        }

        if !self.adrs.is_empty() {
            println!("ADR files to create:");
            for a in &self.adrs {
                println!("  {} (status: {})", a.filename, a.status);
            }
            println!();
        }

        if !self.tests.is_empty() {
            println!("Test criteria files to create:");
            for t in &self.tests {
                println!("  {} (type: {}, adr: {})", t.filename, t.test_type, t.adr_id);
            }
            println!();
        }

        if !self.warnings.is_empty() {
            println!("Warnings:");
            for w in &self.warnings {
                println!("  {}", w);
            }
            println!();
        }

        if !self.conflicts.is_empty() {
            println!("Conflicts:");
            for c in &self.conflicts {
                println!("  {}", c);
            }
            println!();
        }
    }
}
