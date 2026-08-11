//! Rules over what a decision is allocated to, at what tier, by whom.
//!
//! Five of the ten classes: `L001` unallocated, `L003` expired acceptance,
//! `L005` stranded below a raised floor, `L006` an acceptor who is not an
//! accountable actor, `L010` a judgment exercised by one. `L002` and `L004`
//! are raised earlier — an unpriced escape and a below-floor override are
//! unrepresentable in the domain type, so they surface when the wire form is
//! assembled rather than here.

use chrono::NaiveDate;

use crate::finding::{Finding, VerifyClass};
use crate::identity::Identity;
use crate::store::Store;

use super::view::View;

/// `L001` — a decision whose latest version allocates to no store.
///
/// Not to be confused with "allocated, awaiting acceptance", which is a
/// visible status and no finding at all: the gate polices violations, not
/// pendency.
pub fn unallocated(view: &View) -> Vec<Finding> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter(|p| p.allocation.is_none())
        .map(|p| {
            Finding::new(
                VerifyClass::L001,
                &p.decision.to_string(),
                "no allocation — demand is never reduced by leaving it unassigned (§2.1)",
            )
        })
        .collect()
}

/// `L003` — a live acceptance whose expiry has passed.
///
/// §4.4 is explicit that this is a failure, not a warning. Only acceptances
/// signing the *current* version are judged: an acceptance of a superseded
/// version was already invalidated by the hash moving, and reporting it
/// again would be noise on top of a resolved fact.
pub fn expired(view: &View, today: NaiveDate) -> Vec<Finding> {
    view.acceptances
        .iter()
        .map(|a| a.acceptance)
        .filter(|a| !view.is_revoked(a) && view.signs_latest(a))
        .filter(|a| a.is_expired(today))
        .map(|a| {
            let when = a.expires_at.map(|d| d.to_string()).unwrap_or_default();
            Finding::new(
                VerifyClass::L003,
                &a.id.to_string(),
                format!("acceptance of {} expired {when} — re-acceptance is by a named actor, never a carry-over", a.decision),
            )
        })
        .collect()
}

/// `L005` — a member whose effective tier sits below its set's floor.
///
/// Raising a floor is a set-level governed act that re-opens acceptance on
/// every member below the new line. Re-acceptance means a new version
/// pinning the new floor; entries are never grandfathered (§4.2.1).
pub fn stranded(view: &View, store: &Store) -> Vec<Finding> {
    view.latest_versions()
        .filter_map(|v| v.parsed.as_ref())
        .filter_map(|p| store.set(&p.set).map(|s| (p, s)))
        .filter(|(p, s)| p.tolerance.is_stranded_below(s.tolerance_floor))
        .map(|(p, s)| {
            Finding::new(
                VerifyClass::L005,
                &p.decision.to_string(),
                format!(
                    "effective tier {} is below set `{}`'s floor {} — the raise re-opens acceptance at the new floor (§4.2.1)",
                    p.tolerance.effective(),
                    s.id,
                    s.tolerance_floor
                ),
            )
        })
        .collect()
}

/// `L006` — an acceptor that resolves to a model or a CI identity.
///
/// Runs over both places an acceptance is claimed: the formal `Acceptance`
/// actor, and the `accepted_by` that prices an escape. Acceptance is the one
/// act that cannot be delegated (§9.3); route it to a model and the
/// composite becomes pinnable only distributionally, at which point
/// outcome-accountability is unavailable rather than merely absent.
pub fn model_acceptor(view: &View) -> Vec<Finding> {
    let from_acceptances = view.acceptances.iter().map(|a| a.acceptance).filter_map(|a| {
        reject(&a.actor).map(|why| (a.id.to_string(), format!("acceptance actor {why}")))
    });
    let from_escapes = view.latest_versions().filter_map(|v| {
        let parsed = v.parsed.as_ref()?;
        let acceptor = parsed.allocation.as_ref()?.escape_acceptor()?;
        reject(acceptor).map(|why| (parsed.decision.to_string(), format!("escape acceptor {why}")))
    });
    from_acceptances
        .chain(from_escapes)
        .map(|(subject, message)| Finding::new(VerifyClass::L006, &subject, message))
        .collect()
}

/// `L010` — a judgment whose named actor is a model or a CI identity.
///
/// The spec v1.1 amendment. L0 scoped the model-identity rule to acceptors
/// (`L006`), so a *judgment* allocated to a model actor passed — yet a
/// judgment is exercised per run by an accountable actor (§2.1), and a model
/// is pinnable only distributionally. Judged on latest versions only, like
/// every allocation rule.
pub fn model_judge(view: &View) -> Vec<Finding> {
    view.latest_versions()
        .filter_map(|v| {
            let parsed = v.parsed.as_ref()?;
            let actor = parsed.allocation.as_ref()?.judgment_actor()?;
            reject(actor).map(|why| {
                Finding::new(
                    VerifyClass::L010,
                    &parsed.decision.to_string(),
                    format!("judgment actor {why} — a judgment is exercised by a named accountable actor (§2.1, spec v1.1)"),
                )
            })
        })
        .collect()
}

fn reject(actor: &Identity) -> Option<String> {
    actor.model_or_bot_reason()
}
