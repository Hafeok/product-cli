//! Shared fixtures for this crate's unit tests.

use chrono::{DateTime, NaiveDate, Utc};

use crate::acceptance::{Acceptance, AcceptanceScope};
use crate::allocation::AllocationKind;
use crate::changeset::{ChangeSet, DecisionRecord};
use crate::hash::{version_hash, VersionHash};
use crate::id::{AcceptanceId, ChangeSetId, DecisionId};
use crate::identity::Identity;
use crate::set::{DecisionSet, Ground};
use crate::store::{LoggedChangeSet, Store};
use crate::tier::Tier;
use crate::version::VersionRaw;

pub const DEC_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXV";
pub const CS_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXW";
pub const ACC_ULID: &str = "01K2C4YQJ3F8M0PT5W7NZ9RDXX";

pub fn decision_id() -> DecisionId {
    format!("dec:hafeok.ledger/{DEC_ULID}").parse().expect("decision id")
}

pub fn identity(s: &str) -> Identity {
    s.parse().expect("identity")
}

pub fn date(s: &str) -> NaiveDate {
    s.parse().expect("date")
}

pub fn stamp(s: &str) -> DateTime<Utc> {
    s.parse().expect("timestamp")
}

pub fn zero_hash() -> VersionHash {
    format!("sha256:{}", "0".repeat(64)).parse().expect("hash")
}

/// A minimal, well-formed version. Its stored hash is a placeholder; call
/// [`sealed`] to stamp the real one.
pub fn version() -> VersionRaw {
    VersionRaw {
        decision: decision_id(),
        parent: None,
        hash: zero_hash(),
        set: "ledger-design".into(),
        statement: "Monetary amounts use decimal, never double.".into(),
        allocation: Some(AllocationKind::Constraint),
        discharge: vec!["analyzer:DEC001-no-float-money".parse().expect("ref")],
        discharge_stage: None,
        actor: None,
        expectation: None,
        exposure: None,
        accepted_by: None,
        review_by: None,
        tolerance_floor_at_creation: Tier::T1,
        tolerance_override: None,
        based_on: vec!["prd:decision-ledger-prd#4.2.1".parse().expect("basis")],
        supersedes: None,
    }
}

/// Stamp a version with the hash of its own content, as a writer would.
pub fn sealed(mut raw: VersionRaw) -> VersionRaw {
    raw.hash = version_hash(&raw);
    raw
}

pub fn set() -> DecisionSet {
    DecisionSet {
        format: 1,
        id: "ledger-design".into(),
        title: "Decision Ledger — L0 settled design".into(),
        tolerance_floor: Tier::T1,
        ground: Ground::Characterised,
        owner: identity("emk@delegate.dk"),
        created_at: date("2026-08-10"),
        notes: None,
    }
}

pub fn acceptance(version: &VersionRaw) -> Acceptance {
    Acceptance {
        id: format!("acc:{ACC_ULID}").parse().expect("acceptance id"),
        decision: version.decision.clone(),
        version: version.hash.clone(),
        actor: identity("fixture-human@example"),
        at: stamp("2026-08-10T09:20:00Z"),
        scope: AcceptanceScope::Version,
        expires_at: Some(date("2027-08-10")),
        signature: String::new(),
    }
}

pub fn changeset(versions: Vec<VersionRaw>, acceptances: Vec<Acceptance>) -> ChangeSet {
    let decisions = versions
        .iter()
        .map(|v| DecisionRecord {
            id: v.decision.clone(),
            created_at: stamp("2026-08-10T09:14:22Z"),
            created_by: identity("fixture-human@example"),
        })
        .collect();
    ChangeSet {
        format: 1,
        id: format!("cs:{CS_ULID}").parse::<ChangeSetId>().expect("change-set id"),
        created_at: stamp("2026-08-10T09:14:22Z"),
        created_by: identity("fixture-human@example"),
        parents: Vec::new(),
        note: Some("fixture".into()),
        decisions,
        versions,
        acceptances,
        revocations: Vec::new(),
    }
}

/// An in-memory store holding one change-set plus the default set.
pub fn store(changeset: ChangeSet) -> Store {
    Store {
        root: std::path::PathBuf::from("/fixture"),
        dir: std::path::PathBuf::from("/fixture/.decisions"),
        sets: vec![set()],
        log: vec![LoggedChangeSet {
            path: std::path::PathBuf::from(format!("/fixture/.decisions/log/{CS_ULID}.yml")),
            file: changeset,
        }],
        schema_findings: Vec::new(),
    }
}

/// The acceptance-id string, for tests that need to name it.
pub fn acceptance_id() -> AcceptanceId {
    format!("acc:{ACC_ULID}").parse().expect("acceptance id")
}
