use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use vodozemac::{
    Curve25519PublicKey,
    olm::{Account, Message, OlmMessage, PreKeyMessage, Session, SessionConfig},
};

use crate::error_mapper::MapErrorToString;

pub struct SessionInstance {
    pub name: String,
    pub session: Session,
}

pub struct SessionManager {
    sessions: Vec<SessionInstance>,
    current_session: Option<usize>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: vec![],
            current_session: None,
        }
    }

    /// Generate and return public keys for key exchange (identity + one-time key)
    pub fn generate_keys(&mut self, account: &mut Account) -> Result<String, String> {
        // Generate one-time key
        account.generate_one_time_keys(1);
        let one_time_keys = account.one_time_keys();

        // Get the first available one-time key
        let (_, one_time_key) = one_time_keys
            .iter()
            .next()
            .ok_or("Error generating one-time key.")?;

        // Get identity key
        let identity_key = account.identity_keys().curve25519;

        // Encode both keys as base64
        let identity_key_b64 = BASE64_STANDARD_NO_PAD.encode(identity_key.as_bytes());
        let one_time_key_b64 = BASE64_STANDARD_NO_PAD.encode(one_time_key.as_bytes());

        // Bundle: identity_key#one_time_key
        Ok(format!("{}#{}", identity_key_b64, one_time_key_b64))
    }

    /// Parse received keys from the other party
    fn parse_received_keys(
        keys_msg: &str,
    ) -> Result<(Curve25519PublicKey, Curve25519PublicKey), String> {
        let parts: Vec<&str> = keys_msg.split('#').collect();
        if parts.len() != 2 {
            return Err("Invalid keys message format".to_string());
        }

        let identity_key_b64 = parts[0];
        let one_time_key_b64 = parts[1];

        // Decode identity key
        let identity_key_bytes = BASE64_STANDARD_NO_PAD
            .decode(identity_key_b64)
            .map_err_to_string()?;
        let identity_key =
            Curve25519PublicKey::from_slice(&identity_key_bytes).map_err_to_string()?;

        // Decode one-time key
        let one_time_key_bytes = BASE64_STANDARD_NO_PAD
            .decode(one_time_key_b64)
            .map_err_to_string()?;
        let one_time_key =
            Curve25519PublicKey::from_slice(&one_time_key_bytes).map_err_to_string()?;

        Ok((identity_key, one_time_key))
    }

    /// Establish session after key exchange (create outbound session)
    pub fn establish_session(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_msg: &str,
    ) -> Result<(), String> {
        // Generate session config
        let session_config = SessionConfig::version_1();

        // Parse the remote party's keys
        let (remote_identity_key, remote_one_time_key) =
            Self::parse_received_keys(remote_keys_msg)?;

        // Create outbound session using remote keys
        let session = account
            .create_outbound_session(session_config, remote_identity_key, remote_one_time_key)
            .map_err_to_string()?;

        // Mark keys as published
        account.mark_keys_as_published();

        // Store session
        let session_instance = SessionInstance {
            name: name.to_string(),
            session,
        };
        self.sessions.push(session_instance);

        Ok(())
    }

    /// Establish session from the first received message (inbound session)
    pub fn establish_session_from_message(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_msg: &str,
        first_message: &str,
    ) -> Result<(), String> {
        // Generate session config
        let session_config = SessionConfig::version_1();

        // Parse remote keys
        let (remote_identity_key, _) = Self::parse_received_keys(remote_keys_msg)?;

        // Parse the PreKeyMessage from first encrypted message
        let pre_key_message = PreKeyMessage::from_base64(first_message).map_err_to_string()?;

        // Create inbound session
        let session_creation_result = account
            .create_inbound_session(session_config, remote_identity_key, &pre_key_message)
            .map_err_to_string()?;
        let session = session_creation_result.session;

        // Store session
        let session_instance = SessionInstance {
            name: name.to_string(),
            session,
        };
        self.sessions.push(session_instance);

        Ok(())
    }

    pub fn delete_session(&mut self, name: &str) {
        let removed_idx = self.sessions.iter().position(|s| s.name == name);
        self.sessions.retain(|session| session.name != name);

        if let Some(idx) = removed_idx {
            if self.current_session == Some(idx) {
                self.current_session = None;
            } else if let Some(curr) = self.current_session
                && idx < curr
            {
                self.current_session = Some(curr - 1);
            }
        }
    }

    pub fn get_session_names(&self) -> Vec<&str> {
        self.sessions
            .iter()
            .map(|session| session.name.as_str())
            .collect()
    }

    pub fn select_session(&mut self, name: &str) -> Result<(), String> {
        let current_session = self.sessions.iter().position(|s| s.name == name);

        match current_session {
            Some(current_session) => self.current_session = Some(current_session),
            None => return Err(format!("Session '{}' not found", name)),
        }

        Ok(())
    }

    pub fn deselect_session(&mut self) {
        self.current_session = None;
    }

    pub fn get_current_session(&self) -> Option<&SessionInstance> {
        self.current_session.and_then(|idx| self.sessions.get(idx))
    }

    pub fn get_current_session_mut(&mut self) -> Option<&mut SessionInstance> {
        let idx = self.current_session?;
        self.sessions.get_mut(idx)
    }

    pub fn encrypt(&mut self, plaintext: &str) -> Result<String, String> {
        let session_instance = self
            .get_current_session_mut()
            .ok_or("No session selected")?;

        let encrypted = session_instance
            .session
            .encrypt(plaintext)
            .map_err_to_string()?;

        // Convert OlmMessage to base64
        let encrypted_b64 = match encrypted {
            OlmMessage::Normal(msg) => msg.to_base64(),
            OlmMessage::PreKey(msg) => msg.to_base64(),
        };

        Ok(encrypted_b64)
    }

    pub fn decrypt(&mut self, ciphertext_b64: &str) -> Result<String, String> {
        let session_instance = self
            .get_current_session_mut()
            .ok_or("No session selected")?;

        // Try decoding as PreKeyMessage first, then as normal Message
        let olm_message = if let Ok(pre_key_msg) = PreKeyMessage::from_base64(ciphertext_b64) {
            OlmMessage::PreKey(pre_key_msg)
        } else if let Ok(normal_msg) = Message::from_base64(ciphertext_b64) {
            OlmMessage::Normal(normal_msg)
        } else {
            return Err("Failed to decode message from base64".to_string());
        };

        // Decrypt the message
        let plaintext = session_instance
            .session
            .decrypt(&olm_message)
            .map_err_to_string()?;

        String::from_utf8(plaintext).map_err_to_string()
    }
}
