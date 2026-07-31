/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Unit tests for [`SessionManager`] — the per-user collection of Olm sessions
//! (add / select / delete / list) plus export/import used for persistence.
//!
//! A valid session can only be obtained via a real key exchange, so the
//! `add_outbound` helper establishes one per test rather than faking it.

use super::*;

/// Establishes one outbound session named `name` into `mgr`.
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
    add_outbound(&mut mgr, "a");
    add_outbound(&mut mgr, "b");
    assert_eq!(mgr.get_session_names(), vec!["a", "b"]);

    assert!(mgr.get_current_session().is_none());
    mgr.select_session("a").unwrap();
    assert_eq!(mgr.get_current_session().unwrap().name, "a");

    mgr.delete_session("a");
    assert_eq!(mgr.get_session_names(), vec!["b"]);
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
    // Sessions are persisted as encrypted pickles keyed by the user's key;
    // re-importing with the same key must restore them.
    let key = [5u8; 32];
    let mut mgr = SessionManager::new();
    add_outbound(&mut mgr, "s");

    let exported = mgr.export_sessions(&key).unwrap();
    let mut restored = SessionManager::new();
    restored.import_sessions(exported, &key).unwrap();

    assert_eq!(restored.get_session_names(), vec!["s"]);
}

#[test]
fn two_party_conversation_through_managers() {
    // End-to-end exchange driven entirely through the SessionManager API
    // (establish + select + encrypt/decrypt on the active session), the way
    // two real peers talk to each other.
    let mut bazya_acc = Account::new();
    let mut dev_acc = Account::new();
    let bazya_bundle = SessionInstance::generate_keys(&mut bazya_acc).unwrap();
    let dev_bundle = SessionInstance::generate_keys(&mut dev_acc).unwrap();

    let mut bazya = SessionManager::new();
    let mut dev = SessionManager::new();

    // bazya opens the outbound side and sends the empty pre-key message;
    // dev opens the inbound side from it.
    bazya
        .establish_out_session(&mut bazya_acc, "chat", &dev_bundle)
        .unwrap();
    bazya.select_session("chat").unwrap();
    let init = bazya.encrypt("").unwrap();

    dev.establish_in_session(&mut dev_acc, "chat", &bazya_bundle, &init)
        .unwrap();
    dev.select_session("chat").unwrap();

    // Several messages in both directions.
    let m1 = bazya.encrypt("hello").unwrap();
    assert_eq!(dev.decrypt(&m1).unwrap(), "hello");

    let m2 = dev.encrypt("hi back").unwrap();
    assert_eq!(bazya.decrypt(&m2).unwrap(), "hi back");

    let m3 = bazya.encrypt("how are you").unwrap();
    assert_eq!(dev.decrypt(&m3).unwrap(), "how are you");
}
