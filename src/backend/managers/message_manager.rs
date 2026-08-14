/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use colorize::AnsiColor;
use rand::RngExt;

use crate::backend::managers::storage_manager::WorkerHandle;
use crate::backend::objects::message::Message;
use crate::backend::objects::user::User;

/// Encrypts plaintext using the active session and saves to database
pub fn encrypt(
    db_handle: &WorkerHandle,
    user: &mut User,
    plaintext: &str,
) -> Result<String, String> {
    let session_name = user
        .session_manager
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    // Encrypt message for network with OLM
    let net_encrypted = user.encrypt(plaintext)?;

    // Encrypt message for DB with AES-256-GCM
    let db_encrypted = Message::encrypt(&user.encryption_key, &user.name, plaintext)?;

    // Generate random message ID
    let mut rng = rand::rng();
    let message_id = rng.random::<u32>().to_string();

    // Save encrypted message to database
    let db = db_handle.worker();
    db.save_message(&message_id, &session_name, &db_encrypted)?;

    Ok(net_encrypted)
}

/// Decrypts ciphertext using the active session and saves to database
pub fn decrypt(
    db_handle: &WorkerHandle,
    user: &mut User,
    ciphertext_b64: &str,
) -> Result<String, String> {
    let session_name = user
        .session_manager
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    // Decrypt message from network with OLM
    let net_decrypted = user.decrypt(ciphertext_b64)?;

    // Encrypt decrypted message for DB with AES-256-GCM
    let db_encrypted = Message::encrypt(&user.encryption_key, &session_name, &net_decrypted)?;

    // Generate random message ID
    let mut rng = rand::rng();
    let message_id = rng.random::<u32>().to_string();

    // Save encrypted message to database
    let db = db_handle.worker();
    db.save_message(&message_id, &session_name, &db_encrypted)?;

    Ok(net_decrypted)
}

/// Retrieves all messages from a session, sorts by timestamp, and formats them for display
pub fn get_session_messages(db_handle: &WorkerHandle, user: &User) -> Result<String, String> {
    let session_name = user
        .session_manager
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    // Retrieve messages from DB
    let db = db_handle.worker();
    let encrypted_messages = db.get_messages_by_session(&session_name)?;

    // Decrypt all messages
    let mut decrypted_messages = Vec::new();
    for encrypted_bytes in encrypted_messages {
        match Message::decrypt(&user.encryption_key, &encrypted_bytes) {
            Ok(msg) => {
                decrypted_messages.push(msg);
            }
            Err(e) => {
                eprintln!("Failed to decrypt message: {}", e);
            }
        }
    }

    // Sort messages by timestamp in ascending order
    decrypted_messages.sort_by_key(|msg| msg.timestamp);

    // Format each message
    let formatted_messages: Vec<String> = decrypted_messages
        .iter()
        .map(|msg| {
            // Convert Unix timestamp to readable format
            let datetime = chrono::DateTime::<chrono::Utc>::from(
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(msg.timestamp),
            );
            let time_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            // Get plaintext from decrypted data
            let plaintext = String::from_utf8_lossy(&msg.data).to_string();

            format!(
                "{}: {}: {}",
                time_str.b_grey(),
                msg.sender.clone().cyan(),
                plaintext.grey()
            )
        })
        .collect();

    Ok(formatted_messages.join("\n"))
}

/// Returns the current session's messages as (timestamp, sender, plaintext), sorted ascending by time.
pub fn get_session_history(
    db_handle: &WorkerHandle,
    user: &User,
) -> Result<Vec<(u64, String, String)>, String> {
    let session_name = user
        .session_manager
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    let db = db_handle.worker();
    let encrypted_messages = db.get_messages_by_session(&session_name)?;

    let mut messages = Vec::new();
    for encrypted_bytes in encrypted_messages {
        if let Ok(msg) = Message::decrypt(&user.encryption_key, &encrypted_bytes) {
            messages.push(msg);
        }
    }
    messages.sort_by_key(|msg| msg.timestamp);

    Ok(messages
        .into_iter()
        .map(|msg| {
            (
                msg.timestamp,
                msg.sender,
                String::from_utf8_lossy(&msg.data).to_string(),
            )
        })
        .collect())
}
