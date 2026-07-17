use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde_json::{Value, from_value, to_value};
use sha2::{Digest, Sha256};
use vodozemac::olm::{Account, AccountPickle};

use crate::backend::managers::session_manager::SessionManager;
use crate::error_mapper::MapErrorToString;

const SALT_HASH_LENGTH: usize = 16;
const KEY_LENGTH: usize = 32;

// User

pub struct User {
    pub name: String,
    pub account: Account,
    pub session_manager: SessionManager,
    pub encryption_key: [u8; 32],
}

impl User {
    pub fn new(name: &str, password: &str) -> Result<Self, String> {
        let mut account = Account::new();
        account.generate_fallback_key();

        let encryption_key = Self::derive_encryption_key(name, password)?;

        Ok(Self {
            name: name.to_string(),
            session_manager: SessionManager::default(),
            account,
            encryption_key,
        })
    }

    // Persistence

    /// Serializes the user to encrypted JSON
    /// Encrypts the Olm account and session data using the derived encryption key
    pub fn serialize(&self) -> Result<Value, String> {
        let account_pickle = self.encrypt_account()?;
        let sessions_data = self.serialize_sessions()?;

        to_value((self.name.clone(), account_pickle, sessions_data)).map_err_to_string()
    }

    /// Deserializes a user from encrypted JSON
    /// Verifies the password by deriving the encryption key and attempting to decrypt the account data
    pub fn deserialize(json: Value, password: &str) -> Result<Self, String> {
        let (name, encrypted_account, sessions_data): (String, String, Value) =
            from_value(json).map_err_to_string()?;

        let encryption_key = Self::derive_encryption_key(&name, password)?;

        let account = Self::decrypt_account(&encrypted_account, &encryption_key)?;
        let mut session_manager = SessionManager::default();
        session_manager.import_sessions(sessions_data, &encryption_key)?;

        Ok(Self {
            name,
            session_manager,
            account,
            encryption_key,
        })
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

    /// Serializes all sessions using the encryption key
    fn serialize_sessions(&self) -> Result<Value, String> {
        self.session_manager.export_sessions(&self.encryption_key)
    }

    // Cryptography

    /// Derives an encryption key from the user's name and password
    /// Uses the name to generate a deterministic salt via SHA-256 hashing, then applies Argon2 to derive a 32-byte encryption key
    fn derive_encryption_key(name: &str, password: &str) -> Result<[u8; 32], String> {
        let salt = generate_salt_from_name(name)?;
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

/// Generates a deterministic salt from a username using SHA-256
fn generate_salt_from_name(name: &str) -> Result<SaltString, String> {
    let hash = Sha256::digest(name.as_bytes());
    let salt_bytes = &hash[..SALT_HASH_LENGTH];
    let salt_b64 = BASE64_STANDARD_NO_PAD.encode(salt_bytes);

    SaltString::from_b64(&salt_b64)
        .map_err(|e| format!("Failed to create salt from username: {}", e))
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
