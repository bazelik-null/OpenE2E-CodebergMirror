/*
 * Copyright (C) 2026 bazelik-bob
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for SessionManager.
/// (add / select / delete / list) plus export/import used for persistence.
///
/// A valid session can only be obtained via alice real key exchange, so the add_outbound helper establishes one per test rather than faking it.
use super::*;

/// Establishes one outbound session named name into mgr.
fn add_outbound(mgr: &mut SessionManager, name: &str) {
    let mut account = Account::new();
    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
    mgr.establish_out_session(&mut account, name, &peer_bundle)
        .unwrap();
}

#[test]
fn add_list_select_delete() {
    let mut mgr = SessionManager::new();
    add_outbound(&mut mgr, "alice");
    add_outbound(&mut mgr, "bob");
    assert_eq!(mgr.get_session_names(), vec!["alice", "bob"]);

    assert!(mgr.get_current_session().is_none());
    mgr.select_session("alice").unwrap();
    assert_eq!(mgr.get_current_session().unwrap().name, "alice");

    mgr.delete_session("alice");
    assert_eq!(mgr.get_session_names(), vec!["bob"]);
    // Deleting the active session also clears the selection.
    assert!(mgr.get_current_session().is_none());
}

#[test]
fn duplicate_name_rejected() {
    let mut mgr = SessionManager::new();
    add_outbound(&mut mgr, "dup");

    let mut account = Account::new();
    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
    assert!(
        mgr.establish_out_session(&mut account, "dup", &peer_bundle)
            .is_err()
    );
}

#[test]
fn select_unknown_session_fails() {
    let mut mgr = SessionManager::new();
    assert!(mgr.select_session("missing").is_err());
}

#[test]
fn export_import_round_trip() {
    // Sessions are persisted as encrypted pickles keyed by the user's key. Re-importing with the same key must restore them.
    let key = [5u8; 32];
    let mut mgr = SessionManager::new();
    add_outbound(&mut mgr, "session");

    let exported = mgr.export_sessions(&key).unwrap();
    let mut restored = SessionManager::new();
    restored.import_sessions(exported, &key).unwrap();

    assert_eq!(restored.get_session_names(), vec!["session"]);
}

#[test]
fn two_party_conversation_through_managers() {
    // End-to-end exchange driven entirely through the SessionManager API (establish + select + encrypt/decrypt on the active session).
    // Imitate two real peers talk to each other.
    let mut alice_acc = Account::new();
    let mut bob_acc = Account::new();
    let alice_bundle = SessionInstance::generate_keys(&mut alice_acc).unwrap();
    let bob_bundle = SessionInstance::generate_keys(&mut bob_acc).unwrap();

    let mut alice = SessionManager::new();
    let mut bob = SessionManager::new();

    // alice opens the outbound side and sends the empty pre-key message.
    // bob opens the inbound side from it.
    alice
        .establish_out_session(&mut alice_acc, "chat", &bob_bundle)
        .unwrap();
    alice.select_session("chat").unwrap();
    let init = alice.encrypt("").unwrap();

    bob.establish_in_session(&mut bob_acc, "chat", &alice_bundle, &init)
        .unwrap();
    bob.select_session("chat").unwrap();

    // Several messages in both directions.
    let m1 = alice.encrypt("secret1").unwrap();
    assert_eq!(bob.decrypt(&m1).unwrap(), "secret1");

    let m2 = bob.encrypt("secret2").unwrap();
    assert_eq!(alice.decrypt(&m2).unwrap(), "secret2");

    let m3 = alice.encrypt("secret3").unwrap();
    assert_eq!(bob.decrypt(&m3).unwrap(), "secret3");
}
