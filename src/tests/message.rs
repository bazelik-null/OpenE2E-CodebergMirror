/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for Message - the AES-256-GCM envelope used to store chat history encrypted at rest.
///
/// These cover the encrypt/decrypt round-trip and GCM's authenticity guarantee:
/// A wrong key or any tampering with the serialized bytes must be rejected rather than silently returning garbage.
use super::*;

const KEY: [u8; 32] = [7u8; 32];

#[test]
fn encrypt_decrypt_round_trip() {
    let bytes = Message::encrypt(&KEY, "user", "secret".as_bytes()).unwrap();
    let msg = Message::decrypt(&KEY, &bytes).unwrap();

    assert_eq!(msg.sender, "user");
    assert_eq!(String::from_utf8(msg.data).unwrap(), "secret");
}

#[test]
fn empty_plaintext_round_trip() {
    // The session-creation flow encrypts an empty pre-key message, so the empty-plaintext case must survive a round-trip.
    let bytes = Message::encrypt(&KEY, "user", "".as_bytes()).unwrap();
    let msg = Message::decrypt(&KEY, &bytes).unwrap();

    assert!(msg.data.is_empty());
}

#[test]
fn decrypt_with_wrong_key_fails() {
    let bytes = Message::encrypt(&KEY, "user", "secret".as_bytes()).unwrap();
    let wrong_key = [9u8; 32];

    assert!(Message::decrypt(&wrong_key, &bytes).is_err());
}

#[test]
fn tampered_ciphertext_fails() {
    // GCM is authenticated: Dlipping any byte of the envelope must fail the integrity check instead of yielding corrupted plaintext.
    let mut bytes = Message::encrypt(&KEY, "user", "secret".as_bytes()).unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;

    assert!(Message::decrypt(&KEY, &bytes).is_err());
}

#[test]
fn nonce_is_randomized_per_message() {
    // GCM requires a unique nonce per encryption. The implementation uses a random one, so identical plaintext must not produce identical output.
    let a = Message::encrypt(&KEY, "user", "secret".as_bytes()).unwrap();
    let b = Message::encrypt(&KEY, "user", "secret".as_bytes()).unwrap();

    assert_ne!(a, b);
}
