use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde_json::{Error, Value, from_value, to_value};
use sha2::{Digest, Sha256};
use vodozemac::olm::{Account, AccountPickle};

pub struct User {
    pub name: String,
    account: Account,
    key: [u8; 32], // Key for serialization
}

impl User {
    /// Creates new account with keys and unique id
    pub fn new(key_count: usize, name: &str, password: &str) -> Result<User, String> {
        let mut account = Account::new();

        // Generate one time keys for establishing sessions
        account.generate_one_time_keys(key_count);

        // Generate a fallback key (used when out of one time keys)
        account.generate_fallback_key();

        // Get key
        let salt = hash_name_to_salt(name)?;
        let key = derive_key_from_password(password, salt)?;

        Ok(User {
            name: name.to_string(),
            account,
            key,
        })
    }

    /// Serializes user struct to encrypted JSON
    pub fn serialize(&self) -> Result<Value, Error> {
        let pickle = self.account.pickle().encrypt(&self.key);

        to_value((self.name.clone(), pickle.to_string()))
    }

    /// Deserializes user struct from encrypted JSON
    pub fn deserialize(json: Value, password: &str) -> Result<User, String> {
        let serialized_user: (String, String) =
            from_value(json).map_err(|error| error.to_string())?;

        // Get username
        let name = serialized_user.0;

        // Get key
        let salt = hash_name_to_salt(&name)?;
        let key = derive_key_from_password(password, salt)?;

        // Decrypt account data
        let pickle = AccountPickle::from_encrypted(&serialized_user.1, &key)
            .map_err(|error| error.to_string())?;
        let account = Account::from_pickle(pickle);

        Ok(User { name, account, key })
    }
}

/// Hashes the account name to create salt
fn hash_name_to_salt(name: &str) -> Result<SaltString, String> {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();

    let salt_bytes = &hash[..16];
    let salt_b64 = BASE64_STANDARD_NO_PAD.encode(salt_bytes);

    SaltString::from_b64(&salt_b64).map_err(|error| error.to_string())
}

pub fn derive_key_from_password(password: &str, salt: SaltString) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| error.to_string())?;

    // Extract the hash and convert to [u8; 32]
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    Ok(key)
}
