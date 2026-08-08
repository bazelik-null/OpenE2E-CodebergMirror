/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

/// Unit tests for User persistence.
//
/// The account is stored encrypted under a key derived from salt + password (Argon2).
/// These check that a correct password round-trips, a wrong password is rejected, and that key derivation is valid (the salt comes from the data blob).
use super::*;

#[test]
fn serialize_deserialize_round_trip() {
    let user = User::new("user", "password").unwrap();
    let key = user.encryption_key;
    let (name, salt, account_data, sessions) = user.serialize().unwrap();

    let restored = User::deserialize(name, salt, account_data, sessions, "password").unwrap();
    assert_eq!(restored.name, "user");
    assert_eq!(restored.encryption_key, key);
}

#[test]
fn deserialize_with_wrong_password_fails() {
    let user = User::new("user", "password").unwrap();
    let (name, salt, account_data, sessions) = user.serialize().unwrap();

    assert!(User::deserialize(name, salt, account_data, sessions, "!password").is_err());
}
