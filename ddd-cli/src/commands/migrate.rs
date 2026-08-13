//! `ddd migrate-ledger` — the M8 migration of `.ddd` governance records
//! into the decision ledger.
//!
//! Files every decision (incl. the risk-acceptance record), every seam
//! declaration, and the watched-edge ontology question as ledger entries
//! in one declared set — allocated, awaiting the principal's acceptance,
//! never accepted here (M8 ruling 8: ratifications do not auto-convert;
//! manufacturing acceptances during a migration is the falsification the
//! ledger exists to prevent). Writes the permanent id concordance last.

use std::path::PathBuf;

use ledger_core::author::{AddArgs, Author, DeclareArgs};
use product_core::error::{ProductError, Result};

use super::migrate_plan::{plan, Planned, PRINCIPAL};

pub fn run(root: Option<PathBuf>, namespace: String, set: String) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let ddd_dir = repo_root.join(ddd_core::store::STORE_DIR);
    if ddd_core::concordance::load(&ddd_dir).is_some() {
        return Err(ProductError::ConfigError(
            "a concordance already exists — the migration has already run".to_string(),
        ));
    }
    let store = ddd_core::store::load(&ddd_dir);
    let entries = plan(&repo_root, &store).map_err(ProductError::ConfigError)?;
    let mut author = Author::open(&repo_root).map_err(|e| author_err(&e))?;
    declare_set(&mut author, &set)?;
    let mut rows = Vec::new();
    for entry in &entries {
        let ledger_id = file_entry(&mut author, &namespace, &set, entry)?;
        println!("filed {} <- {} ({})", ledger_id, entry.historical, entry.kind);
        rows.push(concordance_row(entry, ledger_id));
    }
    let filed = rows.clone();
    add_manifest_rows(&store, &filed, &mut rows);
    add_stated_dispositions(&mut rows);
    write_concordance(&ddd_dir, rows)?;
    println!(
        "\n{} entries filed into `{set}`, allocated, awaiting acceptance — none accepted \
         here; the queue is the principal's. Concordance written to .ddd/concordance.yaml \
         (permanent).",
        entries.len()
    );
    Ok(())
}

fn declare_set(author: &mut Author, set: &str) -> Result<()> {
    let owner = PRINCIPAL.parse().map_err(ProductError::ConfigError)?;
    author
        .declare(DeclareArgs {
            id: set.to_string(),
            title: "DDD governance — the migrated record of the .ddd store (M8)".to_string(),
            tolerance_floor: ledger_core::tier::Tier::T1,
            ground: ledger_core::set::Ground::Characterised,
            owner: Some(owner),
            notes: Some(
                "Every entry here was migrated from `.ddd/` at ddd M8 (bootstrap form \
                 retired as the record; the .ddd files remain as pinned content artifacts). \
                 Migrated entries land allocated-awaiting-acceptance: a git-recorded \
                 ratification attests a commit, not a signature over a content hash, so \
                 nothing here was accepted on the principal's behalf."
                    .to_string(),
            ),
        })
        .map_err(|e| author_err(&e))?;
    Ok(())
}

/// File one planned entry; returns the minted `dec:` id.
fn file_entry(
    author: &mut Author,
    namespace: &str,
    set: &str,
    entry: &Planned,
) -> Result<String> {
    let applied = author
        .add(AddArgs {
            set: set.to_string(),
            statement: entry.statement.clone(),
            namespace: Some(namespace.to_string()),
            allocation: clone_allocation(&entry.allocation),
            tolerance_override: None,
            based_on: entry.based_on.clone(),
            // The M8 migration states no reopen edges: the three it carried
            // as provisional `watched:` markers were re-decided into real
            // `revisit_if` edges afterwards, as their own filed versions.
            revisit_if: Vec::new(),
            note: Some(entry.note.clone()),
        })
        .map_err(|e| author_err(&e))?;
    applied
        .lines
        .first()
        .and_then(|l| l.split_whitespace().nth(1))
        .map(str::to_string)
        .ok_or_else(|| ProductError::ConfigError("add reported no id".to_string()))
}

/// `AllocationArgs` carries no `Clone`; the migration files each plan
/// once, so a field-by-field copy keeps the core type untouched.
fn clone_allocation(a: &ledger_core::author::AllocationArgs) -> ledger_core::author::AllocationArgs {
    ledger_core::author::AllocationArgs {
        store: a.store,
        discharge: a.discharge.clone(),
        discharge_stage: a.discharge_stage,
        expectation: a.expectation.clone(),
        actor: a.actor.clone(),
        exposure: a.exposure.clone(),
        accepted_by: a.accepted_by.clone(),
        review_by: a.review_by,
    }
}

fn concordance_row(entry: &Planned, ledger_id: String) -> ddd_core::concordance::Row {
    ddd_core::concordance::Row {
        historical: entry.historical.clone(),
        ledger: Some(ledger_id),
        kind: entry.kind.to_string(),
        disposition: None,
    }
}

/// Manifest entries are absorbed as `analyzer:` discharges on their
/// governing decisions; their historical ids resolve to those entries.
fn add_manifest_rows(
    store: &ddd_core::DddStore,
    filed: &[ddd_core::concordance::Row],
    rows: &mut Vec<ddd_core::concordance::Row>,
) {
    for set in &store.manifests {
        for rule in &set.file.rules {
            let Some(decision) = rule.decision.as_deref() else { continue };
            let ledger = filed
                .iter()
                .find(|r| r.historical == decision)
                .and_then(|r| r.ledger.clone());
            rows.push(ddd_core::concordance::Row {
                historical: format!("{}/{}", set.name, rule.rule_id),
                ledger,
                kind: "manifest-entry".to_string(),
                disposition: Some(format!(
                    "absorbed as an analyzer: discharge on its governing decision ({decision})"
                )),
            });
        }
    }
}

/// The two ids created after the concordance was scoped (re-typing
/// handoff): the risk record migrates like any decision (its row is
/// already filed); `DDD-method-07`'s stance is stated here.
fn add_stated_dispositions(rows: &mut Vec<ddd_core::concordance::Row>) {
    rows.push(ddd_core::concordance::Row {
        historical: "DDD-method-07".to_string(),
        ledger: None,
        kind: "claim".to_string(),
        disposition: Some(
            "remains a .ddd claim — claims are the ddd ontology's own and do not migrate; \
             listed explicitly because it was created after the concordance was scoped"
                .to_string(),
        ),
    });
}

fn write_concordance(
    ddd_dir: &std::path::Path,
    rows: Vec<ddd_core::concordance::Row>,
) -> Result<()> {
    let concordance = ddd_core::concordance::Concordance {
        format: 1,
        note: "Permanent id concordance for the M8 ledger migration (ruling 6: the aliases \
               are permanent, not transitional). Historical ids — decisions, the risk \
               record, seams, manifest entries — remain valid forever; `ddd why` resolves \
               either spelling. Claim ids are not re-keyed: claims remain the ddd \
               ontology's own."
            .to_string(),
        rows,
    };
    let yaml = serde_yaml::to_string(&concordance)
        .map_err(|e| ProductError::IoError(e.to_string()))?;
    product_core::fileops::write_file_atomic(
        &ddd_core::concordance::path(ddd_dir),
        &yaml,
    )
    .map_err(|e| ProductError::IoError(e.to_string()))?;
    Ok(())
}

fn author_err(e: &ledger_core::author::AuthorError) -> ProductError {
    use ledger_core::author::AuthorError;
    let msg = match e {
        AuthorError::Refused(findings) => format!(
            "the ledger gate refused the write: {}",
            findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        ),
        AuthorError::Conflict(m) | AuthorError::Usage(m) | AuthorError::Io(m) => m.clone(),
    };
    ProductError::ConfigError(msg)
}
