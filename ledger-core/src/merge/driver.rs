//! The git merge driver's three-way judgment, as a pure function.
//!
//! Git invokes the driver only when both sides changed one file. For the
//! append-only log that is never legitimate — a written log file is never
//! edited, and two different files under one ULID are a collision — so a
//! log-file invocation is always a refusal. A set file admits the two
//! mechanical outcomes (one side unchanged from the ancestor), and refuses
//! the rest: a divergent set change is a judgment, not a text merge.

use crate::set::DecisionSet;

use super::ConflictClass;

/// What the driver should do with one file.
#[derive(Debug, PartialEq, Eq)]
pub enum DriverOutcome {
    /// The current (ours) content stands.
    KeepOurs,
    /// The other side's content stands — write it over ours.
    TakeTheirs,
    /// Judgment required: exit non-zero, conflict preserved.
    Conflict { class: ConflictClass, message: String },
}

/// Three-way merge one `.decisions/` file. `path` is repo-relative.
pub fn three_way(path: &str, base: &str, ours: &str, theirs: &str) -> DriverOutcome {
    if ours == theirs {
        return DriverOutcome::KeepOurs;
    }
    if path.contains("/log/") {
        return log_conflict(path, base);
    }
    if base == ours {
        return DriverOutcome::TakeTheirs;
    }
    if base == theirs {
        return DriverOutcome::KeepOurs;
    }
    DriverOutcome::Conflict {
        class: ConflictClass::DivergentFloor,
        message: divergent_set_message(path, base, ours, theirs),
    }
}

fn log_conflict(path: &str, base: &str) -> DriverOutcome {
    if base.is_empty() {
        DriverOutcome::Conflict {
            class: ConflictClass::UlidCollision,
            message: format!(
                "[ulid-collision] {path}: two different change-sets under one ULID — re-mint one side's change-set id; a log file is written once and never merged"
            ),
        }
    } else {
        DriverOutcome::Conflict {
            class: ConflictClass::EditedLogFile,
            message: format!(
                "[edited-log-file] {path}: a written log file was edited on both sides — the log is append-only; a correction is a new version, a reversal a revocation"
            ),
        }
    }
}

fn divergent_set_message(path: &str, base: &str, ours: &str, theirs: &str) -> String {
    let floor = |text: &str| {
        serde_yaml::from_str::<DecisionSet>(text)
            .map(|s| s.tolerance_floor.to_string())
            .unwrap_or_else(|_| "?".to_string())
    };
    let (b, o, t) = (floor(base), floor(ours), floor(theirs));
    let detail = if o != t {
        format!("floors diverge: base {b}, ours {o}, theirs {t}")
    } else {
        "both sides changed the set differently".to_string()
    };
    format!(
        "[divergent-floor] {path}: {detail} — a set change on both sides is a judgment; `ledger merge --resolve` arbitrates"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SET_T1: &str = "format: 1\nid: s\ntitle: S\ntolerance_floor: T1\nground: characterised\nowner: fixture-human@example\ncreated_at: 2026-08-10\n";
    const SET_T2: &str = "format: 1\nid: s\ntitle: S\ntolerance_floor: T2\nground: characterised\nowner: fixture-human@example\ncreated_at: 2026-08-10\n";
    const SET_T0: &str = "format: 1\nid: s\ntitle: S\ntolerance_floor: T0\nground: characterised\nowner: fixture-human@example\ncreated_at: 2026-08-10\n";

    #[test]
    fn identical_sides_agree_whatever_the_path() {
        let same = three_way(".decisions/log/X.yml", "", "abc", "abc");
        assert_eq!(same, DriverOutcome::KeepOurs);
    }

    #[test]
    fn a_one_sided_set_change_is_mechanical() {
        assert_eq!(three_way(".decisions/sets/s.yml", SET_T1, SET_T1, SET_T2), DriverOutcome::TakeTheirs);
        assert_eq!(three_way(".decisions/sets/s.yml", SET_T1, SET_T2, SET_T1), DriverOutcome::KeepOurs);
    }

    #[test]
    fn a_divergent_floor_is_a_named_judgment_never_a_pick() {
        let out = three_way(".decisions/sets/s.yml", SET_T1, SET_T0, SET_T2);
        let DriverOutcome::Conflict { class, message } = out else { panic!("expected conflict") };
        assert_eq!(class, ConflictClass::DivergentFloor);
        assert!(message.contains("base T1, ours T0, theirs T2"), "{message}");
        assert!(message.contains("merge --resolve"), "{message}");
    }

    #[test]
    fn a_log_file_changed_on_both_sides_refuses_by_class() {
        let collision = three_way(".decisions/log/X.yml", "", "a", "b");
        let DriverOutcome::Conflict { class, .. } = collision else { panic!("conflict") };
        assert_eq!(class, ConflictClass::UlidCollision);
        let edited = three_way(".decisions/log/X.yml", "base", "a", "b");
        let DriverOutcome::Conflict { class, message } = edited else { panic!("conflict") };
        assert_eq!(class, ConflictClass::EditedLogFile);
        assert!(message.contains("append-only"), "{message}");
    }
}
