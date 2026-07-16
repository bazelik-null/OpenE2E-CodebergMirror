use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde_json::{Value, from_value, to_value};
use sha2::{Digest, Sha256};
use vodozemac::olm::{Account, AccountPickle};

use crate::backend::managers::session_manager::SessionManager;
use crate::error_mapper::MapErrorToString;

pub struct User {
    pub name: String,
    pub account: Account,
    pub session_manager: SessionManager,
    key: [u8; 32], // Key for serialization
}

impl User {
    /// Creates new account and unique id
    pub fn new(name: &str, password: &str) -> Result<User, String> {
        let mut account = Account::new();

        // Generate a fallback key (used when out of one time keys)
        account.generate_fallback_key();

        // Get key
        let salt = hash_name_to_salt(name)?;
        let key = derive_key_from_password(password, salt)?;

        Ok(User {
            name: name.to_string(),
            session_manager: SessionManager::default(),
            account,
            key,
        })
    }

    /// Serializes user struct to encrypted JSON
    pub fn serialize(&self) -> Result<Value, String> {
        // Encrypt account
        let pickle = self.account.pickle().encrypt(&self.key);

        // Serialize sessions
        let sessions = self
            .session_manager
            .serialize_sessions(&self.key)
            .map_err_to_string()?;

        // Serialize user
        to_value((self.name.clone(), pickle.to_string(), sessions)).map_err_to_string()
    }

    /// Deserializes user struct from encrypted JSON
    pub fn deserialize(json: Value, password: &str) -> Result<User, String> {
        let serialized_user: (String, String, Value) = from_value(json).map_err_to_string()?;

        // Get username
        let name = serialized_user.0;

        // Get key
        let salt = hash_name_to_salt(&name)?;
        let key = derive_key_from_password(password, salt)?;

        // Decrypt account data
        let pickle = AccountPickle::from_encrypted(&serialized_user.1, &key).map_err_to_string()?;
        let account = Account::from_pickle(pickle);

        // Deserialize sessions
        let mut session_manager = SessionManager::default();
        session_manager.deserialize_sessions(serialized_user.2, &key)?;

        Ok(User {
            name,
            session_manager,
            account,
            key,
        })
    }
}

/// Hashes the account name to create salt
fn hash_name_to_salt(name: &str) -> Result<SaltString, String> {
    // Hash name
    let hash = Sha256::digest(name);

    // Get first 16 bytes and convert to base64
    let salt_bytes = &hash[..16];
    let salt_b64 = BASE64_STANDARD_NO_PAD.encode(salt_bytes);

    // Construct salt from base 64
    SaltString::from_b64(&salt_b64).map_err_to_string()
}

/// Hashes password with Argon2 and given salt
pub fn derive_key_from_password(password: &str, salt: SaltString) -> Result<[u8; 32], String> {
    // Set up Argon2 hasher
    let argon2 = Argon2::default();

    // Hash password
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err_to_string()?;

    // Extract the hash and convert to [u8; 32]
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    Ok(key)
}
