//! `ledger accept` / `ledger revoke` — signing, or unsaying a signature.
//!
//! Acceptance signs the latest version's *hash*, never the id, and the
//! actor is whoever git config says is here (OD-3: no `--as`, one source of
//! truth). A model or CI identity is refused by the same `L006` rule the
//! gate runs; an expired-on-arrival acceptance by the same `L003`. This
//! module never creates an acceptance for anyone but the configured
//! identity, and revocation appends — history keeps the mistake and the
//! correction both.

use chrono::NaiveDate;

use crate::acceptance::{Acceptance, AcceptanceScope, Revocation};
use crate::id::{AcceptanceId, DecisionId};
use crate::verify::view::View;

use super::{Applied, Author, AuthorError};

/// What an acceptance covers.
pub struct AcceptArgs {
    pub decision: DecisionId,
    pub expires_at: Option<NaiveDate>,
}

/// What a revocation unsays, and why.
pub struct RevokeArgs {
    pub acceptance: AcceptanceId,
    pub reason: String,
}

/// The one hash an acceptance may sign: the tip of the parent DAG. A
/// forked chain has no signable tip — signing either side would bless one
/// writer's content over the other's without an arbitration on record.
fn signable_tip(
    view: &View,
    decision: &DecisionId,
) -> Result<crate::hash::VersionHash, AuthorError> {
    let subject = decision.to_string();
    if view.is_forked(&subject) {
        return Err(AuthorError::Conflict(format!(
            "the version chain of {decision} is forked — there is no one latest version to sign; `ledger merge --resolve` arbitrates first"
        )));
    }
    view.latest
        .get(&subject)
        .and_then(|i| view.versions.get(*i))
        .map(|v| v.raw.hash.clone())
        .ok_or_else(|| AuthorError::Usage(format!("{decision} has no filed version to accept")))
}

impl Author {
    /// Sign the latest version of a decision as the configured identity.
    pub fn accept(&mut self, args: AcceptArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let view = View::build(&store);
        let hash = signable_tip(&view, &args.decision)?;
        self.refuse_duplicate(&view, &args.decision, &hash)?;
        let acceptance = Acceptance {
            id: self.mint.mint_id("acc").map_err(AuthorError::Io)?,
            decision: args.decision.clone(),
            version: hash.clone(),
            actor: self.who.clone(),
            at: self.now,
            scope: AcceptanceScope::Version,
            expires_at: args.expires_at,
            signature: String::new(),
        };
        let mut candidate = self.shell(None)?;
        candidate.acceptances.push(acceptance);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied {
            path,
            lines: vec![format!(
                "{} accepted {} of {} — the signature names this exact state",
                self.who,
                hash.short(),
                args.decision
            )],
        })
    }

    /// A second live signature of the same hash by the same actor adds
    /// nothing; refusing it keeps the log an act-record, not an echo.
    fn refuse_duplicate(
        &self,
        view: &View,
        decision: &DecisionId,
        hash: &crate::hash::VersionHash,
    ) -> Result<(), AuthorError> {
        let already = view.acceptances.iter().map(|a| a.acceptance).any(|a| {
            !view.is_revoked(a)
                && a.decision == *decision
                && a.version == *hash
                && a.actor == self.who
                && !a.is_expired(self.today())
        });
        if already {
            return Err(AuthorError::Conflict(format!(
                "{decision} already carries your live acceptance of {} — accepting twice adds nothing",
                hash.short()
            )));
        }
        Ok(())
    }

    /// Unsay a prior acceptance, with the reason on record.
    pub fn revoke(&mut self, args: RevokeArgs) -> Result<Applied, AuthorError> {
        if args.reason.trim().is_empty() {
            return Err(AuthorError::Usage("a revocation carries its reason".to_string()));
        }
        let store = self.load();
        let view = View::build(&store);
        let already_revoked = view.revoked.contains(&args.acceptance.to_string());
        if already_revoked {
            return Err(AuthorError::Conflict(format!(
                "{} is already revoked — one reversal is enough",
                args.acceptance
            )));
        }
        let revocation = Revocation {
            acceptance: args.acceptance.clone(),
            at: self.now,
            by: self.who.clone(),
            reason: args.reason.clone(),
        };
        let mut candidate = self.shell(None)?;
        candidate.revocations.push(revocation);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied {
            path,
            lines: vec![format!("revoked {} — {}", args.acceptance, args.reason)],
        })
    }
}
