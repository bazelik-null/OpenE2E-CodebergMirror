/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use crate::backend::objects::user::{SerializedUserTurple, User};
use crate::backend::services::storage_service::WorkerHandle;
use crate::error_mapper::MapErrorToString;

// SerializedUser

#[derive(Clone)]
pub struct SerializedUser {
    pub name: String,
    pub salt: String,
    pub account_data: String,
    pub sessions: Vec<(String, String)>,
}

impl SerializedUser {
    fn new(
        name: String,
        salt: String,
        account_data: String,
        sessions: Vec<(String, String)>,
    ) -> Self {
        Self {
            name,
            salt,
            account_data,
            sessions,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct UserDataBlob {
    salt: String,
    account_data: String,
}

// UserService

pub struct UserService {
    users: Vec<SerializedUser>,
    current_user: Option<User>,
}

impl UserService {
    pub fn new(db_handle: &WorkerHandle) -> Result<Self, String> {
        let users = Self::load_from_db(db_handle)?;

        Ok(Self {
            users,
            current_user: None,
        })
    }

    // User operations

    /// Creates a new user with the given name and password
    pub fn new_user(&mut self, name: &str, password: &str) -> Result<(), String> {
        if self.user_exists(name) {
            return Err(format!("User '{}' already exists", name));
        }

        let user = User::new(name, password)?;
        let (username, salt, account_data, sessions) = user.serialize()?;

        self.users
            .push(SerializedUser::new(username, salt, account_data, sessions));

        Ok(())
    }

    /// Deletes a user by name
    pub fn delete_user(&mut self, db_handle: &WorkerHandle, name: &str) -> Result<(), String> {
        if self.is_current_user(name) {
            self.current_user = None;
        }

        self.users.retain(|user| user.name != name);

        let db = db_handle.worker();
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

    // Authentication

    /// Authenticates and loads a user by name and password
    pub fn login(&mut self, name: &str, password: &str) -> Result<(), String> {
        let serialized_user = self
            .find_user(name)
            .ok_or_else(|| format!("User '{}' not found", name))?;

        // Deserialize with password verification
        let user = User::deserialize(
            serialized_user.name.clone(),
            serialized_user.salt.clone(),
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

    // Persistence

    /// Syncs the current user to storage and saves all users to disk
    pub fn autosave(&mut self, db_handle: &WorkerHandle) -> Result<(), String> {
        self.sync_current_user_to_storage()?;
        self.save_to_db(db_handle)?;

        Ok(())
    }

    /// Syncs the current user's data back to the users list
    fn sync_current_user_to_storage(&mut self) -> Result<(), String> {
        if let Some(ref current_user) = self.current_user {
            let (username, salt, account_data, sessions) = current_user.serialize()?;

            if let Some(stored_user) = self.users.iter_mut().find(|u| u.name == username) {
                stored_user.salt = salt;
                stored_user.account_data = account_data;
                stored_user.sessions = sessions;
            }
        }

        Ok(())
    }

    /// Saves all users to the database
    fn save_to_db(&self, db_handle: &WorkerHandle) -> Result<(), String> {
        let db = db_handle.worker();

        // Save all users to database
        for user in &self.users {
            // Combine salt and account data into a single blob for storage
            let user_blob = UserDataBlob {
                salt: user.salt.clone(),
                account_data: user.account_data.clone(),
            };
            let data_to_save = serde_json::to_vec(&user_blob)
                .map_err(|e| format!("Failed to serialize user data for saving: {}", e))?;

            // Save combined blob
            db.save_user(&user.name, &data_to_save)?;

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

        // Format: (username, combined_salt_and_account_data_blob)
        let users = db.get_all_users()?;

        for (user_name, user_blob_bytes) in &users {
            // Deserialize the blob to extract salt and account data
            let user_blob: UserDataBlob = serde_json::from_slice(user_blob_bytes)
                .map_err(|e| format!("Failed to deserialize user blob for loading: {}", e))?;

            // Retrieve sessions
            let mut sessions = Vec::new();
            for (key, bytes) in db.get_sessions_by_user(user_name)? {
                let value = String::from_utf8(bytes).map_err_to_string()?;
                sessions.push((key, value));
            }

            // Create SerializedUser using the extracted salt
            let user_result = SerializedUser {
                name: user_name.clone(),
                salt: user_blob.salt,
                account_data: user_blob.account_data,
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
                    user.salt.clone(),
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

        for (username, salt, account_data, sessions) in users_data {
            self.users
                .push(SerializedUser::new(username, salt, account_data, sessions));
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/user_service.rs"]
mod tests;
