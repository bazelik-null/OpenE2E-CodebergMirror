/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use brotli::enc::BrotliEncoderParams;
use colorize::AnsiColor;
use log::debug;
use rand::RngExt;

use crate::backend::objects::message::Message;
use crate::backend::objects::user::User;
use crate::backend::services::storage_service::WorkerHandle;
use crate::error_mapper::MapErrorToString;

/// Encrypts plaintext using the active session and saves to database
/// Compresses message via brotli
/// Pass db_override to override saved in database message
pub fn encrypt(
    db_handle: &WorkerHandle,
    user: &mut User,
    plaintext: &[u8],
    db_override: Option<&[u8]>, // If used saves this in DB instead of message
) -> Result<Vec<u8>, String> {
    let session_name = user
        .session_service
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    // Compress message for network
    let (is_compressed, to_encrypt) = compress(plaintext)?;
    let payload = wrap_compression_flag(is_compressed, &to_encrypt);

    // Encrypt message for network with OLM
    let net_encrypted = user.session_service.encrypt(&payload)?;

    let db_message = match db_override {
        Some(m) => m,
        None => plaintext,
    };

    // Encrypt message for DB with AES-256-GCM
    let db_encrypted = Message::encrypt(&user.encryption_key, &user.name, db_message)?;

    // Generate random message ID
    let mut rng = rand::rng();
    let message_id = rng.random::<u32>().to_string();

    // Save encrypted message to database
    let db = db_handle.worker();
    db.save_message(&message_id, &session_name, &db_encrypted)?;

    Ok(net_encrypted)
}

/// Decrypts ciphertext using the active session and saves to database
/// Decompresses message via brotli
/// Pass db_override to override saved in database message
pub fn decrypt(
    db_handle: &WorkerHandle,
    user: &mut User,
    ciphertext: &[u8],
    db_override: Option<&[u8]>, // If used saves this in DB instead of message
) -> Result<Vec<u8>, String> {
    let session_name = user
        .session_service
        .get_current_session()
        .ok_or_else(|| "No session selected".to_string())?
        .name
        .clone();

    // Decrypt message from network with OLM
    let net_decrypted = user.session_service.decrypt(ciphertext)?;

    // Decompress message from network
    let (was_compressed, payload) = unwrap_compression_flag(&net_decrypted)?;
    let decompressed = if was_compressed {
        decompress(payload)?
    } else {
        payload.to_vec()
    };

    let db_message = match db_override {
        Some(m) => m,
        None => &decompressed,
    };

    // Encrypt decrypted message for DB with AES-256-GCM
    let db_encrypted = Message::encrypt(&user.encryption_key, &session_name, db_message)?;

    // Generate random message ID
    let mut rng = rand::rng();
    let message_id = rng.random::<u32>().to_string();

    // Save encrypted message to database
    let db = db_handle.worker();
    db.save_message(&message_id, &session_name, &db_encrypted)?;

    Ok(decompressed)
}

fn compress(bytes: &[u8]) -> Result<(bool, Vec<u8>), String> {
    // Avoid compression overhead for very small payloads
    const MIN_INPUT: usize = 32;

    if bytes.len() < MIN_INPUT {
        debug!("Using decompressed text");
        return Ok((false, bytes.to_vec()));
    }

    let mut output = Vec::new();
    let params = BrotliEncoderParams::default();

    brotli::BrotliCompress(&mut std::io::Cursor::new(bytes), &mut output, &params)
        .map_err_to_string()?;

    debug!(
        "Compression complete: {} -> {} bytes ({:.1}% ratio)",
        bytes.len(),
        output.len(),
        (output.len() as f64 / bytes.len() as f64) * 100.0
    );

    // Keep ratio <= 100%
    if output.len() <= bytes.len() {
        debug!("Using compressed text");
        Ok((true, output))
    } else {
        debug!("Using decompressed text");
        Ok((false, bytes.to_vec()))
    }
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut decompressor = brotli::Decompressor::new(
        std::io::Cursor::new(bytes),
        4096, // buffer size
    );

    std::io::Read::read_to_end(&mut decompressor, &mut output).map_err_to_string()?;
    debug!(
        "Decompression complete: {} -> {} bytes",
        bytes.len(),
        output.len()
    );
    Ok(output)
}

fn wrap_compression_flag(is_compressed: bool, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + data.len());
    out.push(if is_compressed { 1 } else { 0 });
    out.extend_from_slice(data);
    out
}

fn unwrap_compression_flag(bytes: &[u8]) -> Result<(bool, &[u8]), String> {
    if bytes.is_empty() {
        return Err("Missing compression flag".to_string());
    }
    Ok((bytes[0] == 1, &bytes[1..]))
}

/// Retrieves all messages from a session, sorts by timestamp, and formats them for display
pub fn get_session_messages(db_handle: &WorkerHandle, user: &User) -> Result<String, String> {
    let session_name = user
        .session_service
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
        .session_service
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

#[cfg(test)]
#[path = "../../tests/message_service.rs"]
mod tests;
