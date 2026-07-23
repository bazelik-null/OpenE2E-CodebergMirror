use colorize::AnsiColor;
use rand::RngExt;
use std::time::Duration;

use crate::backend::managers::storage_manager::{BackgroundWorker, WorkerHandle};
use crate::backend::objects::message::Message;
use crate::backend::objects::user::{SerializedUserTurple, User};
use crate::error_mapper::MapErrorToString;

// SerializedUser

#[derive(Clone)]
pub struct SerializedUser {
    pub name: String,
    pub account_data: String,
    pub sessions: Vec<(String, String)>,
}

impl SerializedUser {
    fn new(name: String, account_data: String, sessions: Vec<(String, String)>) -> Self {
        Self {
            name,
            account_data,
            sessions,
        }
    }
}

// UserManager

const STORAGE_FILEPATH: &str = "OpenE2E/storage.db";
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(60);

pub struct UserManager {
    users: Vec<SerializedUser>,
    current_user: Option<User>,
    pub db_handle: WorkerHandle,
}

impl UserManager {
    pub fn new() -> Result<Self, String> {
        let worker = BackgroundWorker::new(AUTOSAVE_INTERVAL, STORAGE_FILEPATH)?;
        let handle = worker.start();

        let users = Self::load_from_db(&handle)?;

        let manager = Self {
            users,
            current_user: None,
            db_handle: handle,
        };

        Ok(manager)
    }

    // User operations

    /// Creates a new user with the given name and password
    pub fn new_user(&mut self, name: &str, password: &str) -> Result<(), String> {
        if self.user_exists(name) {
            return Err(format!("User '{}' already exists", name));
        }

        let user = User::new(name, password)?;
        let (username, account_data, sessions) = user.serialize()?;

        self.users
            .push(SerializedUser::new(username, account_data, sessions));

        Ok(())
    }

    /// Deletes a user by name
    pub fn delete_user(&mut self, name: &str) -> Result<(), String> {
        if self.is_current_user(name) {
            self.current_user = None;
        }

        self.users.retain(|user| user.name != name);

        let db = self.db_handle.worker();

        db.delete_user(name)?;

        Ok(())
    }

    /// Checks if a user exists by name
    fn user_exists(&self, name: &str) -> bool {
        self.users.iter().any(|u| u.name == name)
    }

    /// Retrieves all usernames
    pub fn get_usernames(&self) -> Vec<&str> {
        self.users.iter().map(|user| user.name.as_str()).collect()
    }

    // Messaging

    /// Decrypts ciphertext using the active session
    pub fn encrypt(&mut self, plaintext: &str) -> Result<String, String> {
        let user = self
            .get_current_user_mut()
            .ok_or_else(|| "No user selected".to_string())?;
        let session_name = user
            .session_manager
            .get_current_session()
            .ok_or_else(|| "No session selected".to_string())?
            .name
            .clone();

        // Encrypt message for network with OLM
        let net_encrypted = user.encrypt(plaintext)?;

        // Encrypt message for DB with AES-256-GCM
        let db_encrypted = Message::encrypt(&user.encryption_key, &user.name, plaintext)?;

        // Generate random message ID
        let mut rng = rand::rng();
        let message_id = rng.random::<u32>().to_string();

        // Save encrypted message to database
        let db = self.db_handle.worker();
        db.save_message(&message_id, &session_name, &db_encrypted)?;

        Ok(net_encrypted)
    }

    /// Decrypts ciphertext using the active session and saves to database
    pub fn decrypt(&mut self, ciphertext_b64: &str) -> Result<String, String> {
        let user = self
            .get_current_user_mut()
            .ok_or_else(|| "No user selected".to_string())?;
        let session_name = user
            .session_manager
            .get_current_session()
            .ok_or_else(|| "No session selected".to_string())?
            .name
            .clone();

        // Decrypt message from network with OLM
        let net_decrypted = user.decrypt(ciphertext_b64)?;

        // Encrypt decrypted message for DB with AES-256-GCM
        let db_encrypted = Message::encrypt(&user.encryption_key, &session_name, &net_decrypted)?;

        // Generate random message ID
        let mut rng = rand::rng();
        let message_id = rng.random::<u32>().to_string();

        // Save encrypted message to database
        let db = self.db_handle.worker();
        db.save_message(&message_id, &session_name, &db_encrypted)?;

        Ok(net_decrypted)
    }

    /// Retrieves all messages from the current session, sorts by timestamp, and displays them
    pub fn get_session_messages(&self) -> Result<String, String> {
        // Get session name and encryption key
        let (session_name, encryption_key) = {
            let user = self
                .get_current_user()
                .ok_or_else(|| "No user selected".to_string())?;
            let session_name = user
                .session_manager
                .get_current_session()
                .ok_or_else(|| "No session selected".to_string())?
                .name
                .clone();
            let encryption_key = user.encryption_key;
            (session_name, encryption_key)
        };

        // Retrieve messages from DB
        let db = self.db_handle.worker();
        let encrypted_messages = db.get_messages_by_session(&session_name)?;

        // Decrypt all messages
        let mut decrypted_messages = Vec::new();
        for encrypted_bytes in encrypted_messages {
            match Message::decrypt(&encryption_key, &encrypted_bytes) {
                Ok(msg) => {
                    decrypted_messages.push(msg);
                }
                Err(e) => {
                    eprintln!("Failed to decrypt message: {}", e);
                }
            }
        }

        // Sort messages by timestamp in ascending order
        decrypted_messages.sort_by_key(|msg| msg.timestamp);

        // Format each message
        let formatted_messages: Vec<String> = decrypted_messages
            .iter()
            .map(|msg| {
                // Convert Unix timestamp to readable format
                let datetime = chrono::DateTime::<chrono::Utc>::from(
                    std::time::UNIX_EPOCH + std::time::Duration::from_secs(msg.timestamp),
                );
                let time_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                // Get plaintext from decrypted data
                let plaintext = String::from_utf8_lossy(&msg.data).to_string();

                format!(
                    "{}: {}: {}",
                    time_str.b_grey(),
                    msg.sender.clone().cyan(),
                    plaintext.grey()
                )
            })
            .collect();

        Ok(formatted_messages.join("\n"))
    }

    // Authentication

    /// Authenticates and loads a user by name and password
    pub fn login(&mut self, name: &str, password: &str) -> Result<(), String> {
        let serialized_user = self
            .find_user(name)
            .ok_or_else(|| format!("User '{}' not found", name))?;

        // Deserialize with password verification
        let user = User::deserialize(
            serialized_user.name.clone(),
            serialized_user.account_data.clone(),
            serialized_user.sessions.clone(),
            password,
        )?;
        self.current_user = Some(user);
        Ok(())
    }

    /// Logs out active user
    pub fn logout(&mut self) {
        self.current_user = None;
    }

    /// Checks if a user is currently logged in
    fn is_current_user(&self, name: &str) -> bool {
        self.current_user
            .as_ref()
            .map(|user| user.name == name)
            .unwrap_or(false)
    }

    /// Finds a user by name
    fn find_user(&self, name: &str) -> Option<&SerializedUser> {
        self.users.iter().find(|u| u.name == name)
    }

    // Current User Access

    /// Gets active user
    pub fn get_current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }

    /// Gets mutable active user
    pub fn get_current_user_mut(&mut self) -> Option<&mut User> {
        self.current_user.as_mut()
    }

    // Sessions

    pub fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        let user = self
            .get_current_user_mut()
            .ok_or_else(|| "No user selected".to_string())?;
        let username = user.name.clone();

        // Delete session from manager
        user.session_manager.delete_session(session_id);

        let db = self.db_handle.worker();

        // Delete all messages
        let message_ids = db.get_message_ids_by_session(session_id)?;
        for message_id in message_ids {
            db.delete_message(&message_id, session_id)?;
        }

        // Delete session from DB
        db.delete_session(session_id, &username)
    }

    // Persistence

    /// Syncs the current user to storage and saves all users to disk
    pub fn autosave(&mut self) -> Result<(), String> {
        self.sync_current_user_to_storage()?;
        self.save_to_db()?;

        Ok(())
    }

    /// Syncs the current user's data back to the users list
    fn sync_current_user_to_storage(&mut self) -> Result<(), String> {
        if let Some(ref current_user) = self.current_user {
            let (username, account_data, sessions) = current_user.serialize()?;

            if let Some(stored_user) = self.users.iter_mut().find(|u| u.name == username) {
                stored_user.account_data = account_data;
                stored_user.sessions = sessions;
            }
        }

        Ok(())
    }

    /// Saves all users to the database
    fn save_to_db(&self) -> Result<(), String> {
        let db = self.db_handle.worker();

        // Save all users to database
        for user in &self.users {
            // Save user
            db.save_user(&user.name, user.account_data.as_bytes())?;
            // Save sessions
            for session in &user.sessions {
                db.save_session(&session.0, &user.name, session.1.as_bytes())?;
            }
        }

        Ok(())
    }

    /// Loads all users from the database
    fn load_from_db(db_handle: &WorkerHandle) -> Result<Vec<SerializedUser>, String> {
        let db = db_handle.worker();

        let mut result = Vec::new();

        let users = db.get_all_users()?;

        for user in &users {
            let mut sessions = Vec::new();
            for (key, bytes) in db.get_sessions_by_user(&user.0)? {
                let value = String::from_utf8(bytes).map_err_to_string()?;
                sessions.push((key, value));
            }

            let user_result = SerializedUser {
                name: user.0.clone(),
                account_data: String::from_utf8(user.1.clone()).map_err_to_string()?,
                sessions,
            };

            result.push(user_result);
        }

        Ok(result)
    }

    /// Exports all users to a serializable format
    pub fn export_users(&self) -> Result<Vec<SerializedUserTurple>, String> {
        Ok(self
            .users
            .iter()
            .map(|user| {
                (
                    user.name.clone(),
                    user.account_data.clone(),
                    user.sessions.clone(),
                )
            })
            .collect())
    }

    /// Imports users from a serializable format
    pub fn import_users(&mut self, users_data: Vec<SerializedUserTurple>) -> Result<(), String> {
        self.users.clear();
        self.current_user = None;

        for (username, account_data, sessions) in users_data {
            self.users
                .push(SerializedUser::new(username, account_data, sessions));
        }

        Ok(())
    }

    /// Ensures all data is saved and shut down autosave worker
    pub fn shutdown(mut self) -> Result<(), String> {
        self.autosave()?;
        self.db_handle.graceful_shutdown()
    }
}
