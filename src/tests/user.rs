/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

//! Unit tests for [`User`] persistence.
//!
//! The account is stored encrypted under a key derived from name + password
//! (Argon2). These check that a correct password round-trips, a wrong password
//! is rejected, and that key derivation is deterministic (the salt comes from
//! the username).

use super::*;

#[test]
fn serialize_deserialize_round_trip() {
    let user = User::new("bazya", "rust love").unwrap();
    let key = user.encryption_key;
    let (name, account_data, sessions) = user.serialize().unwrap();

    let restored = User::deserialize(name, account_data, sessions, "rust love").unwrap();
    assert_eq!(restored.name, "bazya");
    assert_eq!(restored.encryption_key, key);
}

#[test]
fn deserialize_with_wrong_password_fails() {
    let user = User::new("bazya", "rust love").unwrap();
    let (name, account_data, sessions) = user.serialize().unwrap();

    assert!(User::deserialize(name, account_data, sessions, "wrong password").is_err());
}

#[test]
fn key_derivation_is_deterministic() {
    // Same name + password => same derived key (the salt is derived from the name).
    let a = User::new("second_developer", "pw").unwrap();
    let b = User::new("second_developer", "pw").unwrap();
    assert_eq!(a.encryption_key, b.encryption_key);
}
