//! Identity cases: address shape, normalisation, the model/bot rejection.

use super::*;

fn id(s: &str) -> Identity {
    s.parse().expect("parse")
}

#[test]
fn a_human_address_is_acceptable() {
    let i = id("emk@delegate.dk");
    assert!(i.is_acceptable(), "{:?}", i.model_or_bot_reason());
}

#[test]
fn the_address_is_normalised_to_lowercase_at_parse() {
    // Hashed content: two casings of one acceptor must not make two hashes.
    assert_eq!(id("Emil@Delegate.DK").as_str(), "emil@delegate.dk");
}

#[test]
fn a_display_name_is_not_an_identity() {
    // OD-3 resolves identity by address; this repo's own `user.name` is
    // "Emil Klein -  Claude Code AI", which is precisely why.
    let err = "Emil Klein -  Claude Code AI".parse::<Identity>().expect_err("not an address");
    assert!(err.contains("whitespace"), "{err}");
}

#[test]
fn an_address_needs_a_local_part_plus_a_domain() {
    assert!("emk".parse::<Identity>().is_err());
    assert!("@delegate.dk".parse::<Identity>().is_err());
    assert!("emk@".parse::<Identity>().is_err());
    assert!("emk@a@b.dk".parse::<Identity>().is_err());
    // A dotless domain is legal, and the fixture identities rely on it.
    assert!("fixture-human@example".parse::<Identity>().is_ok());
}

#[test]
fn github_app_bots_are_rejected() {
    let reason = id("dependabot[bot]@users.noreply.github.com")
        .model_or_bot_reason()
        .expect("rejected");
    assert!(reason.contains("[bot]"), "{reason}");
}

#[test]
fn ci_runner_identities_are_rejected() {
    // OD-3: the never-a-model-identity rule extends to CI/bot identities.
    for addr in ["github-actions@github.com", "ci@example.com", "noreply@example.com"] {
        assert!(id(addr).model_or_bot_reason().is_some(), "{addr} should be rejected");
    }
}

#[test]
fn vendor_no_reply_addresses_are_rejected() {
    let reason = id("noreply@anthropic.com").model_or_bot_reason().expect("rejected");
    assert!(reason.contains("vendor no-reply"), "{reason}");
}

#[test]
fn model_tokens_are_rejected_including_versioned_ones() {
    for addr in [
        "claude@example.com",
        "claude-code@example.com",
        "gpt4@example.com",
        "copilot.agent@example.com",
        "llama3@example.com",
    ] {
        assert!(id(addr).model_or_bot_reason().is_some(), "{addr} should be rejected");
    }
}

#[test]
fn model_tokens_match_whole_tokens_never_substrings() {
    // The false-positive guard: real people are named Claudia and Alain.
    for addr in ["claudia@example.com", "alain@example.com", "magpie@example.com"] {
        assert!(id(addr).is_acceptable(), "{addr} should not be caught");
    }
}

#[test]
fn a_bare_ai_local_part_is_rejected_by_design() {
    // Documented false positive: a human whose address is `ai@` is rejected
    // and must use a fuller address. Accepting a model is the worse error.
    assert!(id("ai@example.com").model_or_bot_reason().is_some());
}

#[test]
fn identities_round_trip_through_serde() {
    let i = id("emk@delegate.dk");
    let json = serde_json::to_string(&i).expect("serialize");
    assert_eq!(json, "\"emk@delegate.dk\"");
    assert_eq!(serde_json::from_str::<Identity>(&json).expect("deserialize"), i);
}
