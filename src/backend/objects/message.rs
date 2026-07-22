use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use cipher::consts::U12;
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a message with metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub timestamp: u64,
    pub sender: String,
    pub nonce: Vec<u8>,
    pub data: Vec<u8>, // Encrypted/decrypted message data
}

impl Message {
    /// Encrypts a message and serializes it to JSON bytes
    pub fn encrypt(key: &[u8; 32], sender: &str, plaintext: &str) -> Result<Vec<u8>, String> {
        // Initialize AES-256-GCM cipher with the provided key
        let cipher = Aes256Gcm::new(key.into());

        // Generate a random 12-byte nonce for GCM mode
        let mut rng = rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::<U12>::from(nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Get current Unix timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Timestamp error: {}", e))?
            .as_secs();

        let encrypted_msg = Message {
            timestamp,
            sender: sender.to_string(),
            nonce: nonce_bytes.to_vec(),
            data: ciphertext,
        };

        // Serialize to JSON bytes
        serde_json::to_vec(&encrypted_msg).map_err(|e| format!("Serialization failed: {}", e))
    }

    /// Decrypts a serialized encrypted message and verifies authenticity
    pub fn decrypt(key: &[u8; 32], bytes: &[u8]) -> Result<Message, String> {
        // Deserialize the JSON bytes
        let mut msg: Message =
            serde_json::from_slice(bytes).map_err(|e| format!("Deserialization failed: {}", e))?;

        // Initialize cipher with the provided key
        let cipher = Aes256Gcm::new(key.into());

        // Convert nonce bytes to array format
        let nonce_array: [u8; 12] = msg
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid nonce length: expected 12 bytes".to_string())?;
        let nonce = Nonce::<U12>::from(nonce_array);

        // Decrypt the ciphertext
        let plaintext_bytes = cipher
            .decrypt(&nonce, msg.data.as_ref())
            .map_err(|e| format!("Decryption failed: {}", e))?;
        msg.data = plaintext_bytes;

        Ok(msg)
    }
}
