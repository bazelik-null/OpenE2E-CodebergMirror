/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Integration tests for [`UserManager`] against a real fjall database.
//!
//! Each test opens the manager on its own temporary directory (via
//! [`UserManager::with_storage_path`]) so they stay isolated and can run in
//! parallel. The focus is persistence: state written before a clean shutdown
//! must reload after "restarting" (reopening) the manager.

use super::*;
use crate::backend::objects::session::SessionInstance;
use vodozemac::olm::Account;

/// A fresh temporary database path. The returned `TempDir` must be kept alive
/// for the duration of the test — dropping it deletes the directory.
fn temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db").to_str().unwrap().to_string();
    (dir, path)
}

#[test]
fn user_persists_across_restart() {
    let (_dir, path) = temp_db();

    // First run: create a user and shut down cleanly (which flushes to disk).
    {
        let mut mgr = UserManager::with_storage_path(&path).unwrap();
        mgr.new_user("bazya", "pw").unwrap();
        mgr.shutdown().unwrap();
    }

    // Second run: the user is still there.
    let mgr = UserManager::with_storage_path(&path).unwrap();
    assert!(mgr.get_usernames().contains(&"bazya"));
    mgr.shutdown().unwrap();
}

#[test]
fn session_persists_across_restart() {
    let (_dir, path) = temp_db();

    // First run: create a user, log in, and establish an outbound session.
    {
        let mut mgr = UserManager::with_storage_path(&path).unwrap();
        mgr.new_user("bazya", "pw").unwrap();
        mgr.login("bazya", "pw").unwrap();

        let mut peer = Account::new();
        let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
        let user = mgr.get_current_user_mut().unwrap();
        user.session_manager
            .establish_out_session(&mut user.account, "demo", &peer_bundle)
            .unwrap();

        mgr.shutdown().unwrap();
    }

    // Second run: after logging back in, the session is restored.
    let mut mgr = UserManager::with_storage_path(&path).unwrap();
    mgr.login("bazya", "pw").unwrap();
    let names = mgr
        .get_current_user()
        .unwrap()
        .session_manager
        .get_session_names();
    assert!(names.contains(&"demo"));
    mgr.shutdown().unwrap();
}

#[test]
fn wrong_password_login_fails() {
    let (_dir, path) = temp_db();

    let mut mgr = UserManager::with_storage_path(&path).unwrap();
    mgr.new_user("bazya", "pw").unwrap();

    assert!(mgr.login("bazya", "wrong").is_err());
    mgr.shutdown().unwrap();
}
