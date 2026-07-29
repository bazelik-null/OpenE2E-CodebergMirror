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
