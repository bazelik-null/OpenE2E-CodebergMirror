/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
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

pub struct SessionManager {
    sessions: Vec<SessionInstance>,
    current_session_name: Option<String>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            current_session_name: None,
        }
    }
    // Key Exchange

    /// Generates identity and one-time keys for key exchange
    ///
    /// Returns a base64-encoded string in the format: identity_key#one_time_key
    pub fn generate_keys(&mut self, account: &mut Account) -> Result<String, String> {
        account.generate_one_time_keys(1);
        let one_time_keys = account.one_time_keys();

        let (_, one_time_key) = one_time_keys
            .iter()
            .next()
            .ok_or_else(|| "Failed to generate one-time key".to_string())?;

        let identity_key = account.identity_keys().curve25519;

        let keys_bundle = Self::encode_keys_bundle(&identity_key, one_time_key);
        Ok(keys_bundle)
    }

    /// Encodes identity and one-time keys as a base64 bundle
    fn encode_keys_bundle(
        identity_key: &Curve25519PublicKey,
        one_time_key: &Curve25519PublicKey,
    ) -> String {
        let identity_b64 = BASE64_STANDARD_NO_PAD.encode(identity_key.as_bytes());
        let one_time_b64 = BASE64_STANDARD_NO_PAD.encode(one_time_key.as_bytes());
        format!("{}{}{}", identity_b64, "#", one_time_b64)
    }

    /// Parses a keys bundle string into identity and one-time public keys.
    fn parse_keys_bundle(
        keys_bundle: &str,
    ) -> Result<(Curve25519PublicKey, Curve25519PublicKey), String> {
        let parts: Vec<&str> = keys_bundle.split("#").collect();
        if parts.len() != 2 {
            return Err(
                "Invalid keys bundle format: expected 'identity_key#one_time_key'".to_string(),
            );
        }

        let identity_key = Self::decode_public_key(parts[0], "identity")?;
        let one_time_key = Self::decode_public_key(parts[1], "one-time")?;

        Ok((identity_key, one_time_key))
    }

    /// Decodes a base64-encoded public key.
    fn decode_public_key(key_b64: &str, key_type: &str) -> Result<Curve25519PublicKey, String> {
        let key_bytes = BASE64_STANDARD_NO_PAD
            .decode(key_b64)
            .map_err(|e| format!("Failed to decode {} key: {}", key_type, e))?;

        Curve25519PublicKey::from_slice(&key_bytes)
            .map_err(|e| format!("Invalid {} key bytes: {}", key_type, e))
    }

    // Session Management

    /// Creates an outbound session
    pub fn establish_out_session(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &str,
    ) -> Result<(), String> {
        let (remote_identity_key, remote_one_time_key) =
            Self::parse_keys_bundle(remote_keys_bundle)?;

        let session_config = SessionConfig::version_1();
        let session = account
            .create_outbound_session(session_config, remote_identity_key, remote_one_time_key)
            .map_err_to_string()?;

        account.mark_keys_as_published();

        self.add_session(SessionInstance {
            name: name.to_string(),
            session,
        });

        Ok(())
    }

    /// Creates an inbound session
    pub fn establish_in_session(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &str,
        first_message_b64: &str,
    ) -> Result<(), String> {
        let (remote_identity_key, _) = Self::parse_keys_bundle(remote_keys_bundle)?;
        let pre_key_message = PreKeyMessage::from_base64(first_message_b64).map_err_to_string()?;

        let session_config = SessionConfig::version_1();
        let session_creation_result = account
            .create_inbound_session(session_config, remote_identity_key, &pre_key_message)
            .map_err_to_string()?;

        self.add_session(SessionInstance {
            name: name.to_string(),
            session: session_creation_result.session,
        });

        Ok(())
    }

    /// Adds a session to the manager
    fn add_session(&mut self, session: SessionInstance) {
        self.sessions.push(session);
    }

    /// Deletes a session by name
    pub fn delete_session(&mut self, name: &str) {
        if self.is_current_session(name) {
            self.current_session_name = None;
        }
        self.sessions.retain(|session| session.name != name);
    }

    /// Retrieves all session names
    pub fn get_session_names(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    /// Selects a session by name as active
    pub fn select_session(&mut self, name: &str) -> Result<(), String> {
        self.find_session(name)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        self.current_session_name = Some(name.to_string());
        Ok(())
    }

    /// Deactivates the active session
    pub fn deselect_session(&mut self) {
        self.current_session_name = None;
    }

    /// Gets the active session
    pub fn get_current_session(&self) -> Option<&SessionInstance> {
        self.current_session_name
            .as_ref()
            .and_then(|name| self.find_session(name))
    }

    /// Gets mutable active session
    pub fn get_current_session_mut(&mut self) -> Option<&mut SessionInstance> {
        let name = self.current_session_name.clone()?;
        self.sessions.iter_mut().find(|s| s.name == name)
    }

    /// Checks if a session name is the current session
    fn is_current_session(&self, name: &str) -> bool {
        self.current_session_name.as_deref() == Some(name)
    }

    /// Finds a session by name
    fn find_session(&self, name: &str) -> Option<&SessionInstance> {
        self.sessions.iter().find(|s| s.name == name)
    }

    // Encryption/Decryption

    /// Encrypts plaintext using the active session
    pub fn encrypt(&mut self, plaintext: &str) -> Result<String, String> {
        let session = self
            .get_current_session_mut()
            .ok_or_else(|| "No session selected".to_string())?;

        let encrypted = session.session.encrypt(plaintext).map_err_to_string()?;
        Ok(Self::encode_olm_message(&encrypted))
    }

    /// Decrypts ciphertext using the active session
    pub fn decrypt(&mut self, ciphertext_b64: &str) -> Result<String, String> {
        let session = self
            .get_current_session_mut()
            .ok_or_else(|| "No session selected".to_string())?;

        let olm_message = Self::decode_olm_message(ciphertext_b64)?;
        let plaintext = session.session.decrypt(&olm_message).map_err_to_string()?;

        String::from_utf8(plaintext).map_err_to_string()
    }

    /// Encodes an OlmMessage to base64
    fn encode_olm_message(message: &OlmMessage) -> String {
        match message {
            OlmMessage::Normal(msg) => msg.to_base64(),
            OlmMessage::PreKey(msg) => msg.to_base64(),
        }
    }

    /// Decodes a base64-encoded message into an OlmMessage
    /// First tries to decode as pre-key, then as regular
    fn decode_olm_message(ciphertext_b64: &str) -> Result<OlmMessage, String> {
        if let Ok(pre_key_msg) = PreKeyMessage::from_base64(ciphertext_b64) {
            return Ok(OlmMessage::PreKey(pre_key_msg));
        }

        if let Ok(normal_msg) = Message::from_base64(ciphertext_b64) {
            return Ok(OlmMessage::Normal(normal_msg));
        }

        Err("Failed to decode message from base64".to_string())
    }

    // Persistence

    /// Exports all sessions as a Vec of tuples (name, encrypted_pickle)
    pub fn export_sessions(&self, key: &[u8; 32]) -> Result<Vec<(String, String)>, String> {
        self.sessions
            .iter()
            .map(|session| {
                let encrypted_pickle = session.serialize(key)?;
                Ok((session.name.clone(), encrypted_pickle))
            })
            .collect()
    }

    /// Imports all sessions from a Vec of tuples (name, encrypted_pickle)
    pub fn import_sessions(
        &mut self,
        sessions: Vec<(String, String)>,
        key: &[u8; 32],
    ) -> Result<(), String> {
        self.sessions = sessions
            .into_iter()
            .map(|(name, encrypted_pickle_str)| {
                SessionInstance::deserialize(name, encrypted_pickle_str, key)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}
