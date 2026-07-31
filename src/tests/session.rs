/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Unit tests for [`SessionInstance`] — the per-peer Olm session (vodozemac).
//!
//! The core guarantee is a working manual key exchange: two accounts publish
//! key bundles, establish an outbound/inbound pair, and can then exchange
//! messages in both directions. Also covers pickle serialize/deserialize, which
//! is how a session survives being persisted to and reloaded from the database.

use super::*;

/// Builds a paired (outbound, inbound) session exactly the way the app does:
/// both peers publish key bundles, the outbound side sends an empty pre-key
/// message, and the inbound side is created from it.
///
/// Returns `(bazya_outbound, second_developer_inbound)`.
fn established_pair() -> (SessionInstance, SessionInstance) {
    let mut bazya = Account::new();
    let mut second_developer: Account = Account::new();

    let bazya_bundle = SessionInstance::generate_keys(&mut bazya).unwrap();
    let second_developer_bundle = SessionInstance::generate_keys(&mut second_developer).unwrap();

    let mut bazya_out =
        SessionInstance::create_outbound(&mut bazya, "s", &second_developer_bundle).unwrap();
    let init = bazya_out.encrypt("").unwrap();
    let second_developer_in =
        SessionInstance::create_inbound(&mut second_developer, "s", &bazya_bundle, &init).unwrap();

    (bazya_out, second_developer_in)
}

#[test]
fn generate_keys_returns_identity_and_one_time() {
    // The bundle is `identity_key#one_time_key`; the peer needs both halves.
    let mut account = Account::new();
    let bundle = SessionInstance::generate_keys(&mut account).unwrap();

    assert_eq!(bundle.split('#').count(), 2);
}

#[test]
fn round_trip_both_directions() {
    let (mut bazya, mut second_developer) = established_pair();

    let ct = bazya.encrypt("hi second_developer").unwrap();
    assert_eq!(
        second_developer.decrypt(&ct).unwrap(),
        "hi second_developer"
    );

    let ct2 = second_developer.encrypt("hi bazya").unwrap();
    assert_eq!(bazya.decrypt(&ct2).unwrap(), "hi bazya");
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
    // A reloaded (unpickled) session must keep its ratchet state and continue
    // talking to the peer — this is what makes persisted sessions usable.
    let (mut bazya, mut second_developer) = established_pair();

    let ct = bazya.encrypt("first").unwrap();
    assert_eq!(second_developer.decrypt(&ct).unwrap(), "first");

    let key = [3u8; 32];
    let pickle = bazya.serialize(&key).unwrap();
    let mut restored = SessionInstance::deserialize("s".to_string(), pickle, &key).unwrap();

    let ct2 = restored.encrypt("after restore").unwrap();
    assert_eq!(second_developer.decrypt(&ct2).unwrap(), "after restore");
}

#[test]
fn deserialize_with_wrong_key_fails() {
    let (bazya, _second_developer) = established_pair();
    let pickle = bazya.serialize(&[1u8; 32]).unwrap();

    assert!(SessionInstance::deserialize("s".to_string(), pickle, &[2u8; 32]).is_err());
}
