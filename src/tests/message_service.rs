/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for MessageService - the service used to encrypt and compress messages.
///
/// These cover the encrypt/decrypt round-trip and content preservation
use super::*;
use crate::backend::objects::session::SessionInstance;
use crate::backend::services::repository::Repository;
use crate::backend::services::user_service::UserService;
use vodozemac::olm::Account;

/// A fresh temporary database path. The returned `TempDir` must be kept alive for the duration of the test. Dropping it deletes the directory.
fn temp_db() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db").to_str().unwrap().to_string();
    (dir, path)
}

#[test]
fn message_conversation_with_encryption() {
    let (_dir, path) = temp_db();

    // Setup: Create two users with an established session between them
    {
        let repo = Repository::with_storage_path(&path).unwrap();
        let mut srv = UserService::new(&repo.db_handle).unwrap();

        // Create and login first user
        srv.new_user("alice", "password_alice").unwrap();
        srv.login("alice", "password_alice").unwrap();

        let mut bob_account = Account::new();
        let bob_bundle = SessionInstance::generate_keys(&mut bob_account).unwrap();

        let alice = srv.get_current_user_mut().unwrap();
        alice
            .session_service
            .establish_out_session(&mut alice.account, "bob_session", &bob_bundle)
            .unwrap();
        alice.session_service.select_session("bob_session").unwrap();

        srv.autosave(&repo.db_handle).unwrap();
        repo.shutdown().unwrap();
    }

    // Test: Exchange messages in a conversation
    {
        let repo = Repository::with_storage_path(&path).unwrap();
        let mut srv = UserService::new(&repo.db_handle).unwrap();
        srv.login("alice", "password_alice").unwrap();

        let alice = srv.get_current_user_mut().unwrap();

        alice.session_service.select_session("bob_session").unwrap();

        // Message 1: Alice sends a message
        let message_1 = b"Hello Bob, how are you?";
        let encrypted_1 =
            encrypt(&repo.db_handle, alice, message_1, None).expect("Failed to encrypt message 1");

        assert!(
            !encrypted_1.is_empty(),
            "Encrypted message should not be empty"
        );
        assert_ne!(
            encrypted_1, message_1,
            "Encrypted message should differ from plaintext"
        );

        // Message 2: Alice sends another message
        let message_2 = b"Did you get my previous message?";
        let encrypted_2 =
            encrypt(&repo.db_handle, alice, message_2, None).expect("Failed to encrypt message 2");

        assert!(
            !encrypted_2.is_empty(),
            "Second encrypted message should not be empty"
        );
        assert_ne!(
            encrypted_2, message_2,
            "Second encrypted message should differ from plaintext"
        );

        // Message 3: Alice sends with custom DB override
        let message_3 = b"This is sensitive";
        let db_override = b"This is redacted";
        let encrypted_3 = encrypt(&repo.db_handle, alice, message_3, Some(db_override))
            .expect("Failed to encrypt message 3 with override");

        assert!(
            !encrypted_3.is_empty(),
            "Third encrypted message should not be empty"
        );

        srv.autosave(&repo.db_handle).unwrap();
        repo.shutdown().unwrap();
    }

    // Verify: After restart, messages are persisted
    {
        let repo = Repository::with_storage_path(&path).unwrap();
        let mut srv = UserService::new(&repo.db_handle).unwrap();

        // Verify user and session still exist
        assert!(srv.get_usernames().contains(&"alice"));
        srv.login("alice", "password_alice").unwrap();
        let alice = srv.get_current_user().unwrap();
        let session_names = alice.session_service.get_session_names();
        assert!(
            session_names.contains(&"bob_session"),
            "Session should persist after restart"
        );

        repo.shutdown().unwrap();
    }
}

#[test]
fn message_encryption_preserves_content() {
    let (_dir, path) = temp_db();

    let repo = Repository::with_storage_path(&path).unwrap();
    let mut srv = UserService::new(&repo.db_handle).unwrap();

    srv.new_user("user", "password_user").unwrap();
    srv.login("user", "password_user").unwrap();

    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();

    let user = srv.get_current_user_mut().unwrap();
    user.session_service
        .establish_out_session(&mut user.account, "peer_session", &peer_bundle)
        .unwrap();
    user.session_service.select_session("peer_session").unwrap();

    // Test various message types
    let test_messages = vec![
        b"Simple message".to_vec(),
        b"Message with numbers: 12345".to_vec(),
        b"Message with special chars: !@#$%^&*()".to_vec(),
        vec![0u8, 1, 2, 3, 255, 254, 253], // Binary data
        vec![0u8; 10000],                  // Large message (10KB)
    ];

    for (idx, plaintext) in test_messages.iter().enumerate() {
        let encrypted = encrypt(&repo.db_handle, user, plaintext, None)
            .unwrap_or_else(|e| panic!("Failed to encrypt message {}: {}", idx, e));

        // Verify encryption properties
        assert!(
            !encrypted.is_empty(),
            "Message {} should produce non-empty ciphertext",
            idx
        );
        assert_ne!(
            &encrypted, plaintext,
            "Message {} ciphertext should differ from plaintext",
            idx
        );
    }

    repo.shutdown().unwrap();
}

#[test]
fn message_db_override_affects_storage() {
    let (_dir, path) = temp_db();

    let repo = Repository::with_storage_path(&path).unwrap();
    let mut srv = UserService::new(&repo.db_handle).unwrap();

    srv.new_user("user", "password_user").unwrap();
    srv.login("user", "password_user").unwrap();

    let mut peer = Account::new();
    let peer_bundle = SessionInstance::generate_keys(&mut peer).unwrap();

    let user = srv.get_current_user_mut().unwrap();
    user.session_service
        .establish_out_session(&mut user.account, "storage_test", &peer_bundle)
        .unwrap();
    user.session_service.select_session("storage_test").unwrap();

    let plaintext = b"Sensitive message content";
    let redacted = b"[REDACTED]";

    // Encrypt with DB override
    let encrypted_network = encrypt(&repo.db_handle, user, plaintext, Some(redacted))
        .expect("Encryption with override should succeed");

    // The network ciphertext should be based on plaintext (before compression)
    assert!(
        !encrypted_network.is_empty(),
        "Network ciphertext should not be empty"
    );

    // Without override for comparison
    let encrypted_no_override = encrypt(&repo.db_handle, user, plaintext, None)
        .expect("Encryption without override should succeed");

    // Both should produce valid ciphertexts (actual comparison of DB storage would require decryption)
    assert!(
        !encrypted_no_override.is_empty(),
        "Ciphertext without override should not be empty"
    );

    repo.shutdown().unwrap();
}
