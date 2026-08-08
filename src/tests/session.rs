/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for [`SessionInstance`] — the per-peer Olm session (vodozemac).
///
/// The core guarantee is a working manual key exchange:
/// Two accounts publish key bundles, establish an outbound/inbound pair, and can then exchange messages in both directions.
/// Also covers pickle serialize/deserialize, which is how a session survives being persisted to and reloaded from the database.
use super::*;

/// Builds a paired (outbound, inbound) session exactly the way the app does:
/// Both peers publish key bundles, the outbound side sends an empty pre-key message, and the inbound side is created from it.
///
/// Returns (alice_outbound, bob_inbound).
fn established_pair() -> (SessionInstance, SessionInstance) {
    let mut alice = Account::new();
    let mut bob: Account = Account::new();

    let alice_bundle = SessionInstance::generate_keys(&mut alice).unwrap();
    let bob_bundle = SessionInstance::generate_keys(&mut bob).unwrap();

    let mut alice_out = SessionInstance::create_outbound(&mut alice, "s", &bob_bundle).unwrap();
    let init = alice_out.encrypt("").unwrap();
    let bob_in = SessionInstance::create_inbound(&mut bob, "s", &alice_bundle, &init).unwrap();

    (alice_out, bob_in)
}

#[test]
fn generate_keys_returns_identity_and_one_time() {
    // The bundle is `identity_key#one_time_key`. The peer needs both halves.
    let mut account = Account::new();
    let bundle = SessionInstance::generate_keys(&mut account).unwrap();

    assert_eq!(bundle.split('#').count(), 2);
}

#[test]
fn round_trip_both_directions() {
    let (mut alice, mut bob) = established_pair();

    let ct = alice.encrypt("secret1").unwrap();
    assert_eq!(bob.decrypt(&ct).unwrap(), "secret1");

    let ct2 = bob.encrypt("secret2").unwrap();
    assert_eq!(alice.decrypt(&ct2).unwrap(), "secret2");
}

#[test]
fn create_inbound_rejects_malformed_bundle() {
    let mut acc = Account::new();
    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
    let mut out = SessionInstance::create_outbound(&mut acc, "s", &peer_bundle).unwrap();
    let init = out.encrypt("").unwrap();

    assert!(SessionInstance::create_inbound(&mut peer, "s", "not-a-bundle", &init).is_err());
}

#[test]
fn serialize_deserialize_preserves_session() {
    // A reloaded (unpickled) session must keep its ratchet state and continue talking to the peer.
    let (mut alice, mut bob) = established_pair();

    let ct = alice.encrypt("first").unwrap();
    assert_eq!(bob.decrypt(&ct).unwrap(), "first");

    let key = [3u8; 32];
    let pickle = alice.serialize(&key).unwrap();
    let mut restored = SessionInstance::deserialize("s".to_string(), pickle, &key).unwrap();

    let ct2 = restored.encrypt("after restore").unwrap();
    assert_eq!(bob.decrypt(&ct2).unwrap(), "after restore");
}

#[test]
fn deserialize_with_wrong_key_fails() {
    let (alice, _bob) = established_pair();
    let pickle = alice.serialize(&[1u8; 32]).unwrap();

    assert!(SessionInstance::deserialize("s".to_string(), pickle, &[2u8; 32]).is_err());
}
