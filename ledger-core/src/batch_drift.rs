//! Naming what moved when a confirmed manifest no longer matches the store.
//!
//! A refusal that says only "the selection changed" tells a principal to
//! re-read from scratch; one that says *which* entry entered, left, or moved
//! tells them whether the change touches what they just read. Naming it
//! needs the earlier selection, which the manifest hash alone cannot give
//! back — so it is recovered rather than stored.
//!
//! The recovery is sound because the log is append-only: every state this
//! store has passed through is a prefix of its change-sets in ULID order.
//! [`locate`] replays those prefixes until one hashes to the manifest that
//! was confirmed, which reconstructs the exact selection the principal read
//! — no cache, no scratch file, nothing to go stale. When no prefix matches,
//! the refusal says so plainly instead of guessing.

use chrono::NaiveDate;
use serde::Serialize;

use crate::identity::Identity;
use crate::show::{screens, NoText, Selector};
use crate::store::Store;

use super::{plan, Plan};

/// One difference between the selection that was read and the one in front
/// of the run now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub decision: String,
    pub what: String,
}

/// The selection a manifest was taken over, recovered from the log's own
/// prefixes. `None` when no prefix hashes to it — the manifest was read
/// against a different selector, a different identity, or a store that has
/// since been rewritten rather than appended to.
pub fn locate(
    store: &Store,
    selector: &Selector,
    actor: &Identity,
    today: NaiveDate,
    manifest: &str,
) -> Option<Plan> {
    (0..store.log.len()).rev().find_map(|k| {
        let prefix = truncated(store, k);
        let candidate = plan(&screens(&prefix, selector, today, &NoText), selector, actor, today);
        (candidate.manifest == manifest).then_some(candidate)
    })
}

/// The store as it stood after its first `k` change-sets. Sets are carried
/// whole: a set file is edited in place rather than appended, so it has no
/// prefix history to replay, and it never changes which pairs a selector
/// resolves to.
fn truncated(store: &Store, k: usize) -> Store {
    Store {
        root: store.root.clone(),
        dir: store.dir.clone(),
        sets: store.sets.clone(),
        log: store.log.iter().take(k).cloned().collect(),
        schema_findings: Vec::new(),
    }
}

/// What moved between two selections, in decision-id order.
pub fn changes(before: &Plan, after: &Plan) -> Vec<Change> {
    let mut out: Vec<Change> = Vec::new();
    for old in &before.members {
        match after.members.iter().find(|n| n.decision == old.decision) {
            None => out.push(Change {
                decision: old.decision.clone(),
                what: "left the selection".to_string(),
            }),
            Some(new) if new.version != old.version => out.push(Change {
                decision: old.decision.clone(),
                what: format!(
                    "a new version was filed — the hash you read, {}, is no longer the tip; it is now {}",
                    short(&old.version),
                    short(&new.version)
                ),
            }),
            Some(new) if new.standing != old.standing => out.push(Change {
                decision: old.decision.clone(),
                what: format!("would now be {} rather than {}", new.standing, old.standing),
            }),
            Some(_) => {}
        }
    }
    for new in &after.members {
        if !before.members.iter().any(|o| o.decision == new.decision) {
            out.push(Change {
                decision: new.decision.clone(),
                what: format!("entered the selection, unread, as {}", new.standing),
            });
        }
    }
    out.sort_by(|a, b| a.decision.cmp(&b.decision));
    out
}

fn short(hash: &str) -> String {
    let hex = hash.strip_prefix("sha256:").unwrap_or(hash);
    hex.get(..12).unwrap_or(hex).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::Member;

    fn member(decision: &str, version: &str, standing: &'static str) -> Member {
        Member {
            decision: decision.to_string(),
            version: format!("sha256:{}", version.repeat(64 / version.len().max(1))),
            group: "individual".to_string(),
            state: "awaiting-acceptance".to_string(),
            statement_head: "x".to_string(),
            standing,
            because: None,
        }
    }

    fn of(members: Vec<Member>) -> Plan {
        Plan {
            selector: "group:individual".to_string(),
            actor: "fixture-human@example".to_string(),
            manifest: crate::batch::manifest("group:individual", "fixture-human@example", &members),
            members,
        }
    }

    #[test]
    fn an_entry_that_appeared_is_named_as_unread() {
        let before = of(vec![member("dec:a", "a", "signable")]);
        let after = of(vec![member("dec:a", "a", "signable"), member("dec:b", "b", "signable")]);
        let moved = changes(&before, &after);
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].decision, "dec:b");
        assert!(moved[0].what.contains("entered the selection, unread"), "{moved:?}");
    }

    #[test]
    fn a_revised_member_names_both_hashes() {
        let before = of(vec![member("dec:a", "a", "signable")]);
        let after = of(vec![member("dec:a", "b", "signable")]);
        let moved = changes(&before, &after);
        assert_eq!(moved.len(), 1);
        assert!(moved[0].what.contains("no longer the tip"), "{moved:?}");
    }

    #[test]
    fn a_standing_that_moved_is_named() {
        let before = of(vec![member("dec:a", "a", "signable")]);
        let after = of(vec![member("dec:a", "a", "already-signed")]);
        let moved = changes(&before, &after);
        assert_eq!(moved[0].what, "would now be already-signed rather than signable");
    }

    #[test]
    fn a_member_that_left_is_named() {
        let before = of(vec![member("dec:a", "a", "signable"), member("dec:b", "b", "signable")]);
        let after = of(vec![member("dec:a", "a", "signable")]);
        let moved = changes(&before, &after);
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].what, "left the selection");
    }

    #[test]
    fn an_unchanged_selection_reports_nothing() {
        let before = of(vec![member("dec:a", "a", "signable")]);
        assert!(changes(&before, &before).is_empty());
    }
}
