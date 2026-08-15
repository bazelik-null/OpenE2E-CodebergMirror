/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use vodozemac::{
    Curve25519PublicKey,
    olm::{Account, Message, OlmMessage, PreKeyMessage, Session, SessionConfig, SessionPickle},
};

use crate::error_mapper::MapErrorToString;

pub struct SessionInstance {
    pub name: String,
    pub session: Session,
}

impl SessionInstance {
    // Key Exchange

    /// Generates identity and one-time keys for key exchange
    /// Returns a base64-encoded string in the format: identity_key#one_time_key
    pub fn generate_keys(account: &mut Account) -> Result<Vec<u8>, String> {
        account.generate_one_time_keys(1);
        let one_time_keys = account.one_time_keys();

        let (_, one_time_key) = one_time_keys
            .iter()
            .next()
            .ok_or_else(|| "Failed to generate one-time key".to_string())?;

        let identity_key = account.identity_keys().curve25519;

        Self::encode_keys_bundle(&identity_key, one_time_key)
    }

    /// Bundle format (big-endian):
    /// u16(identity_len) || identity_bytes || u16(one_time_len) || one_time_bytes
    fn encode_keys_bundle(
        identity_key: &Curve25519PublicKey,
        one_time_key: &Curve25519PublicKey,
    ) -> Result<Vec<u8>, String> {
        const MAX_U16: usize = u16::MAX as usize;

        let id = identity_key.as_bytes();
        let otk = one_time_key.as_bytes();

        // Public keys should be small, but we enforce bounds for safety
        if id.len() > MAX_U16 {
            return Err(format!("identity key too large: {} bytes", id.len()));
        }
        if otk.len() > MAX_U16 {
            return Err(format!("one-time key too large: {} bytes", otk.len()));
        }

        let mut out = Vec::with_capacity(2 + id.len() + 2 + otk.len());
        out.extend_from_slice(&(id.len() as u16).to_be_bytes());
        out.extend_from_slice(id);
        out.extend_from_slice(&(otk.len() as u16).to_be_bytes());
        out.extend_from_slice(otk);
        Ok(out)
    }

    fn parse_keys_bundle(
        bundle: &[u8],
    ) -> Result<(Curve25519PublicKey, Curve25519PublicKey), String> {
        let mut i = 0usize;

        // Read identity key
        let id_len = read_u16(bundle, &mut i, "identity length")? as usize;
        let id_bytes = read_bytes(bundle, &mut i, id_len, "identity key bytes")?;

        // Read one-time key
        let ot_len = read_u16(bundle, &mut i, "one-time length")? as usize;
        let ot_bytes = read_bytes(bundle, &mut i, ot_len, "one-time key bytes")?;

        // Reject trailing bytes
        if i != bundle.len() {
            return Err("Invalid bundle: trailing bytes after keys".to_string());
        }

        // Parse the public key
        let identity_key = Curve25519PublicKey::from_slice(id_bytes)
            .map_err(|e| format!("Invalid identity key bytes: {}", e))?;

        let one_time_key = Curve25519PublicKey::from_slice(ot_bytes)
            .map_err(|e| format!("Invalid one-time key bytes: {}", e))?;

        Ok((identity_key, one_time_key))
    }

    // Session Creation

    /// Creates an outbound session
    pub fn create_outbound(
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &[u8],
    ) -> Result<Self, String> {
        let (remote_identity_key, remote_one_time_key) =
            Self::parse_keys_bundle(remote_keys_bundle)?;

        let session_config = SessionConfig::version_1();
        let session = account
            .create_outbound_session(session_config, remote_identity_key, remote_one_time_key)
            .map_err_to_string()?;

        account.mark_keys_as_published();

        Ok(SessionInstance {
            name: name.to_string(),
            session,
        })
    }

    /// Creates an inbound session
    pub fn create_inbound(
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &[u8],
        first_message: &[u8],
    ) -> Result<Self, String> {
        let (remote_identity_key, _) = Self::parse_keys_bundle(remote_keys_bundle)?;
        let pre_key_message = PreKeyMessage::from_bytes(first_message).map_err_to_string()?;

        let session_config = SessionConfig::version_1();
        let session_creation_result = account
            .create_inbound_session(session_config, remote_identity_key, &pre_key_message)
            .map_err_to_string()?;

        Ok(SessionInstance {
            name: name.to_string(),
            session: session_creation_result.session,
        })
    }

    // Encryption/Decryption

    /// Encrypts plaintext using this session
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let encrypted = self.session.encrypt(plaintext).map_err_to_string()?;
        Ok(Self::encode_olm_message(&encrypted))
    }

    /// Decrypts ciphertext using this session
    pub fn decrypt(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let olm_message = Self::decode_olm_message(ciphertext)?;
        self.session.decrypt(&olm_message).map_err_to_string()
    }

    /// Encodes an OlmMessage to bytes
    fn encode_olm_message(message: &OlmMessage) -> Vec<u8> {
        match message {
            OlmMessage::Normal(msg) => msg.to_bytes(),
            OlmMessage::PreKey(msg) => msg.to_bytes(),
        }
    }

    /// Decodes a byte message into an OlmMessage
    /// First tries to decode as pre-key, then as regular
    fn decode_olm_message(ciphertext: &[u8]) -> Result<OlmMessage, String> {
        if let Ok(pre_key_msg) = PreKeyMessage::from_bytes(ciphertext) {
            return Ok(OlmMessage::PreKey(pre_key_msg));
        }

        if let Ok(normal_msg) = Message::from_bytes(ciphertext) {
            return Ok(OlmMessage::Normal(normal_msg));
        }

        Err("Failed to decode message from bytes".to_string())
    }

    // Persistence

    /// Serializes session to encrypted bytes
    pub fn serialize(&self, key: &[u8; 32]) -> Result<String, String> {
        let pickle = self.session.pickle();
        let encrypted_pickle = pickle.encrypt(key);
        Ok(encrypted_pickle.to_string())
    }

    /// Deserializes session from encrypted bytes
    pub fn deserialize(
        name: String,
        encrypted_pickle_str: String,
        key: &[u8; 32],
    ) -> Result<Self, String> {
        let pickle =
            SessionPickle::from_encrypted(&encrypted_pickle_str, key).map_err_to_string()?;

        let session = Session::from_pickle(pickle);

        Ok(SessionInstance { name, session })
    }
}

fn read_u16(buf: &[u8], i: &mut usize, field: &str) -> Result<u16, String> {
    if buf.len() < *i + 2 {
        return Err(format!("Truncated bundle while reading {}", field));
    }
    let v = u16::from_be_bytes([buf[*i], buf[*i + 1]]);
    *i += 2;
    Ok(v)
}

fn read_bytes<'a>(
    buf: &'a [u8],
    i: &mut usize,
    len: usize,
    field: &str,
) -> Result<&'a [u8], String> {
    if buf.len() < *i + len {
        return Err(format!(
            "Truncated bundle while reading {} (need {}, have {})",
            field,
            len,
            buf.len().saturating_sub(*i)
        ));
    }
    let out = &buf[*i..*i + len];
    *i += len;
    Ok(out)
}

#[cfg(test)]
#[path = "../../tests/session.rs"]
mod tests;
