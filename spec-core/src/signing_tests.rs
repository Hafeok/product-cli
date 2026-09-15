use super::*;

pub(crate) fn key_pair() -> (SigningKey, TrustKey) {
    let (secret, public) = generate();
    let signing = signing_key_from_hex(&secret).expect("generated key parses");
    let trust = TrustKey {
        form: TRUST_KEY_FORM.into(),
        id: "emil-2026".into(),
        principal: "emil@example.com".parse().expect("identity parses"),
        algorithm: "ed25519".into(),
        public_key: public,
        // Adopted at the start of time, so a test act counts as
        // post-adoption unless it deliberately says otherwise.
        added_at: DateTime::UNIX_EPOCH,
    };
    (signing, trust)
}

#[test]
fn a_signature_verifies_under_its_own_key() {
    let (signing, trust) = key_pair();
    let signature = sign("sha256:aa", &signing);
    let principal = trust.principal.clone();
    assert!(verifies("sha256:aa", &signature, &principal, &[trust]));
}

#[test]
fn a_signature_does_not_verify_over_a_different_digest() {
    let (signing, trust) = key_pair();
    let signature = sign("sha256:aa", &signing);
    let principal = trust.principal.clone();
    assert!(!verifies("sha256:bb", &signature, &principal, &[trust]));
}

#[test]
fn a_signature_does_not_verify_under_another_principals_key() {
    let (signing, mine) = key_pair();
    let (_, theirs) = key_pair();
    let signature = sign("sha256:aa", &signing);

    let other_principal: Identity = "someone-else@example.com".parse().expect("identity parses");
    assert!(!verifies("sha256:aa", &signature, &other_principal, &[mine, theirs]));
}

#[test]
fn a_key_bound_to_someone_else_does_not_vouch_for_this_principal() {
    let (signing, mut trust) = key_pair();
    let signature = sign("sha256:aa", &signing);
    let claimed: Identity = "emil@example.com".parse().expect("identity parses");
    trust.principal = "impostor@example.com".parse().expect("identity parses");
    assert!(!verifies("sha256:aa", &signature, &claimed, &[trust]));
}

#[test]
fn an_empty_trust_root_verifies_nothing() {
    let (signing, trust) = key_pair();
    let signature = sign("sha256:aa", &signing);
    let principal = trust.principal.clone();
    assert!(!verifies("sha256:aa", &signature, &principal, &[]));
}

#[test]
fn a_malformed_signature_is_refused_rather_than_panicking() {
    let (_, trust) = key_pair();
    for bad in ["", "zz", "aa", "not-hex-at-all", "abc"] {
        assert!(!verifies("sha256:aa", bad, &trust.principal, std::slice::from_ref(&trust)));
    }
}

#[test]
fn a_key_of_an_unknown_algorithm_cannot_verify() {
    let (_, mut trust) = key_pair();
    trust.algorithm = "rsa".into();
    assert!(trust.verifying_key().is_none());
}

#[test]
fn the_signed_subject_is_domain_separated() {
    assert!(subject("sha256:aa").starts_with(SIGNATURE_FORM.as_bytes()));
    assert_ne!(subject("sha256:aa"), b"sha256:aa".to_vec());
}

#[test]
fn hex_round_trips_and_refuses_what_is_not_hex() {
    assert_eq!(decode_hex(&encode_hex(&[0, 255, 16])), Some(vec![0, 255, 16]));
    assert!(decode_hex("abc").is_none(), "odd length");
    assert!(decode_hex("").is_none(), "empty");
    assert!(decode_hex("zz").is_none(), "not hex");
}
