/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Integration tests for DatabaseService.
///
/// Each test opens its own temporary database directory, so they are isolated and safe to run in parallel.
/// These verify the basic CRUD paths and the user -> sessions / session -> messages indexes.
use super::*;

/// Opens a DatabaseService on a fresh temporary directory. The TempDir must be kept alive for the test's duration (dropping it deletes the directory).
fn temp_db() -> (tempfile::TempDir, DatabaseService) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db").to_str().unwrap().to_string();
    let db = DatabaseService::new(&path).unwrap();
    (dir, db)
}

#[test]
fn user_save_get_delete() {
    let (_dir, db) = temp_db();

    db.save_user("user", b"account-blob").unwrap();
    assert_eq!(db.get_user("user").unwrap(), b"account-blob".to_vec());

    db.delete_user("user").unwrap();
    assert!(db.get_user("user").is_err());
}

#[test]
fn get_all_users_returns_everything() {
    let (_dir, db) = temp_db();

    db.save_user("user", b"a").unwrap();
    db.save_user("second_user", b"b").unwrap();

    assert_eq!(db.get_all_users().unwrap().len(), 2);
}

#[test]
fn sessions_are_indexed_by_user() {
    let (_dir, db) = temp_db();

    db.save_session("s1", "user", b"pickle1").unwrap();
    db.save_session("s2", "user", b"pickle2").unwrap();

    let sessions = db.get_sessions_by_user("user").unwrap();
    assert_eq!(sessions.len(), 2);
}

#[test]
fn messages_are_indexed_by_session() {
    let (_dir, db) = temp_db();

    db.save_message("m1", "chat", b"enc1").unwrap();
    db.save_message("m2", "chat", b"enc2").unwrap();

    assert_eq!(db.get_messages_by_session("chat").unwrap().len(), 2);
    assert_eq!(db.get_message_ids_by_session("chat").unwrap().len(), 2);
}

#[test]
fn missing_keys_return_empty_or_error() {
    let (_dir, db) = temp_db();

    // No sessions/messages recorded for an unknown parent => empty lists.
    assert!(db.get_sessions_by_user("nobody").unwrap().is_empty());
    assert!(db.get_messages_by_session("nochat").unwrap().is_empty());
    // A missing user is an error, not empty bytes.
    assert!(db.get_user("nobody").is_err());
}
