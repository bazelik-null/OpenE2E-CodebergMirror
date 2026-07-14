use argon2::password_hash::Salt;
use argon2::{Argon2, PasswordHasher};
use serde_json::{Error, Value, from_value, to_value};
use std::str::FromStr;
use uuid::Uuid;
use vodozemac::olm::{Account, AccountPickle};

pub struct User {
    pub uid: Uuid,
    account: Account,
    key: [u8; 32], // Key for serialization
}

impl User {
    /// Creates new account with keys and unique id
    pub fn new(key_count: usize, password: &str) -> Result<User, String> {
        let mut account = Account::new();

        // Generate one time keys for establishing sessions
        account.generate_one_time_keys(key_count);

        // Generate a fallback key (used when out of one time keys)
        account.generate_fallback_key();

        // Generate unique id
        let uid = Uuid::new_v4();

        // Get salt
        let uid_str = uid.simple().to_string();
        let salt = Salt::from_b64(&uid_str).map_err(|error| error.to_string())?;

        // Get key
        let key = derive_key_from_password(password, salt)?;

        Ok(User { uid, account, key })
    }

    /// Serializes user struct to encrypted JSON
    pub fn serialize(&self) -> Result<Value, Error> {
        let pickle = self.account.pickle().encrypt(&self.key);

        to_value((self.uid.to_string(), pickle.to_string()))
    }

    /// Deserializes user struct from encrypted JSON
    pub fn deserialize(json: Value, password: &str) -> Result<User, String> {
        let serialized_user: (String, String) =
            from_value(json).map_err(|error| error.to_string())?;

        // Get unique id
        let uid = Uuid::from_str(&serialized_user.0).map_err(|error| error.to_string())?;

        // Get salt
        let uid_str = uid.simple().to_string();
        let salt = Salt::from_b64(&uid_str).map_err(|error| error.to_string())?;

        // Get encryption key
        let key = derive_key_from_password(password, salt)?;

        // Decrypt account data
        let pickle = AccountPickle::from_encrypted(&serialized_user.1, &key)
            .map_err(|error| error.to_string())?;
        let account = Account::from_pickle(pickle);

        Ok(User { uid, account, key })
    }
}

pub fn derive_key_from_password(password: &str, salt: Salt) -> Result<[u8; 32], String> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|error| error.to_string())?;

    // Extract the hash and convert to [u8; 32]
    let hash_bytes = password_hash.hash.ok_or("No hash generated")?;
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    Ok(key)
}
