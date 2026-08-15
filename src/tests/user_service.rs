/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Integration tests for UserService
///
/// Each test opens the service on its own temporary directory via Repository and UserService so they stay isolated and can run in parallel.
/// The focus is persistence: state written before a clean shutdown must reload after "restarting" (reopening) the service.
use super::*;
use crate::backend::objects::session::SessionInstance;
use crate::backend::services::repository::Repository;
use vodozemac::olm::Account;

/// A fresh temporary database path. The returned `TempDir` must be kept alive for the duration of the test. Dropping it deletes the directory.
fn temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db").to_str().unwrap().to_string();
    (dir, path)
}

#[test]
fn user_persists_across_restart() {
    let (_dir, path) = temp_db();

    // First run: Create a user and shut down cleanly (which flushes to disk).
    {
        let repo = Repository::with_storage_path(&path).unwrap();
        let mut srv = UserService::new(&repo.db_handle).unwrap();
        srv.new_user("user", "password").unwrap();
        srv.autosave(&repo.db_handle).unwrap();
        repo.shutdown().unwrap();
    }

    // Second run: The user is still there.
    let repo = Repository::with_storage_path(&path).unwrap();
    let srv = UserService::new(&repo.db_handle).unwrap();
    assert!(srv.get_usernames().contains(&"user"));
    repo.shutdown().unwrap();
}

#[test]
fn session_persists_across_restart() {
    let (_dir, path) = temp_db();

    // First run: Create a user, log in, and establish an outbound session.
    {
        let repo = Repository::with_storage_path(&path).unwrap();
        let mut srv = UserService::new(&repo.db_handle).unwrap();
        srv.new_user("user", "password").unwrap();
        srv.login("user", "password").unwrap();

        let mut peer = Account::new();
        let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();
        let user = srv.get_current_user_mut().unwrap();
        user.session_service
            .establish_out_session(&mut user.account, "test", &peer_bundle)
            .unwrap();

        srv.autosave(&repo.db_handle).unwrap();
        repo.shutdown().unwrap();
    }

    // Second run: After logging back in, the session is restored.
    let repo = Repository::with_storage_path(&path).unwrap();
    let mut srv = UserService::new(&repo.db_handle).unwrap();
    srv.login("user", "password").unwrap();
    let names = srv
        .get_current_user()
        .unwrap()
        .session_service
        .get_session_names();
    assert!(names.contains(&"test"));
    repo.shutdown().unwrap();
}

#[test]
fn wrong_password_login_fails() {
    let (_dir, path) = temp_db();

    let repo = Repository::with_storage_path(&path).unwrap();
    let mut srv = UserService::new(&repo.db_handle).unwrap();
    srv.new_user("user", "password").unwrap();

    assert!(srv.login("user", "!password").is_err());
    repo.shutdown().unwrap();
}
