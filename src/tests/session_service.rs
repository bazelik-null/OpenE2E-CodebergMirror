/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for SessionService.
/// (add / select / delete / list) plus export/import used for persistence.
///
/// A valid session can only be obtained via alice real key exchange, so the add_outbound helper establishes one per test rather than faking it.
use super::*;

/// Establishes one outbound session named name into srv.
fn add_outbound(srv: &mut SessionService, name: &str) {
    let mut account = Account::new();
    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
    srv.establish_out_session(&mut account, name, &peer_bundle)
        .unwrap();
}

#[test]
fn add_list_select_delete() {
    let mut srv = SessionService::new();
    add_outbound(&mut srv, "alice");
    add_outbound(&mut srv, "bob");
    assert_eq!(srv.get_session_names(), vec!["alice", "bob"]);

    assert!(srv.get_current_session().is_none());
    srv.select_session("alice").unwrap();
    assert_eq!(srv.get_current_session().unwrap().name, "alice");

    srv.delete_session("alice");
    assert_eq!(srv.get_session_names(), vec!["bob"]);
    // Deleting the active session also clears the selection.
    assert!(srv.get_current_session().is_none());
}

#[test]
fn duplicate_name_rejected() {
    let mut srv = SessionService::new();
    add_outbound(&mut srv, "dup");

    let mut account = Account::new();
    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
    assert!(
        srv.establish_out_session(&mut account, "dup", &peer_bundle)
            .is_err()
    );
}

#[test]
fn select_unknown_session_fails() {
    let mut srv = SessionService::new();
    assert!(srv.select_session("missing").is_err());
}

#[test]
fn export_import_round_trip() {
    // Sessions are persisted as encrypted pickles keyed by the user's key. Re-importing with the same key must restore them.
    let key = [5u8; 32];
    let mut srv = SessionService::new();
    add_outbound(&mut srv, "session");

    let exported = srv.export_sessions(&key).unwrap();
    let mut restored = SessionService::new();
    restored.import_sessions(exported, &key).unwrap();

    assert_eq!(restored.get_session_names(), vec!["session"]);
}

#[test]
fn two_party_conversation_through_services() {
    // End-to-end exchange driven entirely through the SessionService API (establish + select + encrypt/decrypt on the active session).
    // Imitate two real peers talk to each other.
    let mut alice_acc = Account::new();
    let mut bob_acc = Account::new();
    let alice_bundle = SessionInstance::generate_keys(&mut alice_acc).unwrap();
    let bob_bundle = SessionInstance::generate_keys(&mut bob_acc).unwrap();

    let mut alice = SessionService::new();
    let mut bob = SessionService::new();

    // alice opens the outbound side and sends the empty pre-key message.
    // bob opens the inbound side from it.
    alice
        .establish_out_session(&mut alice_acc, "chat", &bob_bundle)
        .unwrap();
    alice.select_session("chat").unwrap();
    let init = alice.encrypt("".as_bytes()).unwrap();

    bob.establish_in_session(&mut bob_acc, "chat", &alice_bundle, &init)
        .unwrap();
    bob.select_session("chat").unwrap();

    // Several messages in both directions.
    let m1 = alice.encrypt("secret1".as_bytes()).unwrap();
    assert_eq!(bob.decrypt(&m1).unwrap(), "secret1".as_bytes());

    let m2 = bob.encrypt("secret2".as_bytes()).unwrap();
    assert_eq!(alice.decrypt(&m2).unwrap(), "secret2".as_bytes());

    let m3 = alice.encrypt("secret3".as_bytes()).unwrap();
    assert_eq!(bob.decrypt(&m3).unwrap(), "secret3".as_bytes());
}
