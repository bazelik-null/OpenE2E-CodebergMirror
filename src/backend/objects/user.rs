/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHasher};
use vodozemac::olm::{Account, AccountPickle};

use crate::backend::managers::session_manager::SessionManager;
use crate::error_mapper::MapErrorToString;

const KEY_LENGTH: usize = 32;

pub type SerializedUserTurple = (String, String, String, Vec<(String, String)>);

// User

pub struct User {
    pub name: String,
    pub salt: String,
    pub account: Account,
    pub session_manager: SessionManager,
    pub encryption_key: [u8; KEY_LENGTH],
}

impl User {
    pub fn new(name: &str, password: &str) -> Result<Self, String> {
        let mut account = Account::new();
        account.generate_fallback_key();

        let salt = generate_random_salt();
        let salt_b64 = salt.to_string();
        let encryption_key = Self::derive_encryption_key(salt, password)?;

        Ok(Self {
            name: name.to_string(),
            salt: salt_b64,
            session_manager: SessionManager::default(),
            account,
            encryption_key,
        })
    }

    // Persistence

    /// Serializes the user to encrypted format
    /// Format: (username, salt, encrypted_account, sessions_vec)
    pub fn serialize(&self) -> Result<SerializedUserTurple, String> {
        let account_pickle = self.encrypt_account()?;
        let sessions_data = self.serialize_sessions()?;

        Ok((
            self.name.clone(),
            self.salt.clone(),
            account_pickle,
            sessions_data,
        ))
    }

    /// Deserializes a user from encrypted format
    /// Format: (username, salt, encrypted_account, sessions_vec)
    pub fn deserialize(
        username: String,
        salt_string: String,
        encrypted_account: String,
        sessions_data: Vec<(String, String)>,
        password: &str,
    ) -> Result<Self, String> {
        let salt = SaltString::from_b64(&salt_string).map_err_to_string()?;
        let encryption_key = Self::derive_encryption_key(salt, password)?;

        let account = Self::decrypt_account(&encrypted_account, &encryption_key)?;
        let mut session_manager = SessionManager::default();
        session_manager.import_sessions(sessions_data, &encryption_key)?;

        Ok(Self {
            name: username,
            salt: salt_string,
            session_manager,
            account,
            encryption_key,
        })
    }

    /// Serializes all sessions using the encryption key
    fn serialize_sessions(&self) -> Result<Vec<(String, String)>, String> {
        self.session_manager.export_sessions(&self.encryption_key)
    }

    /// Encrypts the account pickle using the encryption key
    fn encrypt_account(&self) -> Result<String, String> {
        let pickle = self.account.pickle().encrypt(&self.encryption_key);
        Ok(pickle.to_string())
    }

    /// Decrypts the account pickle using the encryption key
    fn decrypt_account(encrypted_pickle: &str, key: &[u8; 32]) -> Result<Account, String> {
        let pickle = AccountPickle::from_encrypted(encrypted_pickle, key).map_err_to_string()?;
        Ok(Account::from_pickle(pickle))
    }

    // Cryptography

    /// Derives an encryption key from the user's name and password
    /// Uses provided random salt, then applies Argon2 to derive a 32-byte encryption key
    fn derive_encryption_key(salt: SaltString, password: &str) -> Result<[u8; KEY_LENGTH], String> {
        derive_key_from_password(password, salt)
    }

    // Messaging

    /// Decrypts ciphertext using the active session
    pub fn encrypt(&mut self, plaintext: &str) -> Result<String, String> {
        self.session_manager.encrypt(plaintext)
    }

    /// Decrypts ciphertext
    pub fn decrypt(&mut self, ciphertext_b64: &str) -> Result<String, String> {
        self.session_manager.decrypt(ciphertext_b64)
    }
}

// Utilities

/// Generates a random salt.
fn generate_random_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}

/// Derives a 32-byte encryption key from a password using Argon2.
pub fn derive_key_from_password(password: &str, salt: SaltString) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err_to_string()?;

    let hash_bytes = password_hash
        .hash
        .ok_or_else(|| "Argon2 failed to generate hash".to_string())?;

    let mut key = [0u8; KEY_LENGTH];
    key.copy_from_slice(&hash_bytes.as_bytes()[..KEY_LENGTH]);

    Ok(key)
}

#[cfg(test)]
#[path = "../../tests/user.rs"]
mod tests;
