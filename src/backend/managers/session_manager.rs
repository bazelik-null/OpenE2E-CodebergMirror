use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use serde_json::Value;
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
    /// Serializes session to encrypted JSON
    pub fn serialize(&self, key: &[u8; 32]) -> Result<Value, String> {
        // Pickle the session
        let pickle = self.session.pickle();

        // Encrypt the pickle
        let encrypted_pickle = pickle.encrypt(key);

        // Serialize to JSON
        serde_json::to_value((self.name.clone(), encrypted_pickle.to_string())).map_err_to_string()
    }

    /// Deserializes session from encrypted JSON
    pub fn deserialize(json: Value, key: &[u8; 32]) -> Result<Self, String> {
        // Parse the serialized data
        let (name, encrypted_pickle_str): (String, String) =
            serde_json::from_value(json).map_err_to_string()?;

        // Decrypt the pickle
        let pickle =
            SessionPickle::from_encrypted(&encrypted_pickle_str, key).map_err_to_string()?;

        // Restore the session from pickle
        let session = Session::from_pickle(pickle);

        Ok(SessionInstance { name, session })
    }
}

pub struct SessionManager {
    sessions: Vec<SessionInstance>,
    current_session: Option<String>,
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
        // Logout if deleted session is equal to current
        if self.current_session.as_deref() == Some(name) {
            self.current_session = None;
        }

        self.sessions.retain(|session| session.name != name);
    }

    pub fn get_session_names(&self) -> Vec<&str> {
        self.sessions
            .iter()
            .map(|session| session.name.as_str())
            .collect()
    }

    pub fn select_session(&mut self, name: &str) -> Result<(), String> {
        if self.sessions.iter().any(|s| s.name == name) {
            self.current_session = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Session '{}' not found", name))
        }
    }

    pub fn deselect_session(&mut self) {
        self.current_session = None;
    }

    pub fn get_current_session(&self) -> Option<&SessionInstance> {
        self.current_session
            .as_ref()
            .and_then(|name| self.sessions.iter().find(|s| &s.name == name))
    }

    pub fn get_current_session_mut(&mut self) -> Option<&mut SessionInstance> {
        let name = self.current_session.clone()?;
        self.sessions.iter_mut().find(|s| s.name == name)
    }

    /// Encrypt message using current session
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

    /// Decrypt message using current session
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

    /// Serializes all sessions to encrypted JSON array
    pub fn serialize_sessions(&self, key: &[u8; 32]) -> Result<Value, String> {
        let serialized_sessions: Result<Vec<Value>, String> = self
            .sessions
            .iter()
            .map(|session| session.serialize(key))
            .collect();

        serde_json::to_value(serialized_sessions?).map_err_to_string()
    }

    /// Deserializes all sessions from encrypted JSON array
    pub fn deserialize_sessions(&mut self, json: Value, key: &[u8; 32]) -> Result<(), String> {
        let serialized_sessions: Vec<Value> = serde_json::from_value(json).map_err_to_string()?;

        self.sessions = serialized_sessions
            .into_iter()
            .map(|session_json| SessionInstance::deserialize(session_json, key))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}
