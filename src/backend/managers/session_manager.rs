/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use crate::backend::objects::session::SessionInstance;
use vodozemac::olm::Account;

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

    // Session Management

    /// Creates an outbound session and adds it to the manager
    pub fn establish_out_session(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &str,
    ) -> Result<(), String> {
        let session = SessionInstance::create_outbound(account, name, remote_keys_bundle)?;
        self.add_session(session)
    }

    /// Creates an inbound session and adds it to the manager
    pub fn establish_in_session(
        &mut self,
        account: &mut Account,
        name: &str,
        remote_keys_bundle: &str,
        first_message_b64: &str,
    ) -> Result<(), String> {
        let session =
            SessionInstance::create_inbound(account, name, remote_keys_bundle, first_message_b64)?;
        self.add_session(session)
    }

    /// Adds a session instance to the manager
    pub fn add_session(&mut self, session: SessionInstance) -> Result<(), String> {
        if self.session_exists(&session.name) {
            return Err(format!("Session '{}' already exists", session.name));
        }

        self.sessions.push(session);

        Ok(())
    }

    /// Deletes a session by name
    pub fn delete_session(&mut self, name: &str) {
        if self.is_current_session(name) {
            self.current_session_name = None;
        }
        self.sessions.retain(|session| session.name != name);
    }

    /// Checks if a session exists by name
    fn session_exists(&self, name: &str) -> bool {
        self.sessions.iter().any(|s| s.name == name)
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

        session.encrypt(plaintext)
    }

    /// Decrypts ciphertext using the active session
    pub fn decrypt(&mut self, ciphertext_b64: &str) -> Result<String, String> {
        let session = self
            .get_current_session_mut()
            .ok_or_else(|| "No session selected".to_string())?;

        session.decrypt(ciphertext_b64)
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
