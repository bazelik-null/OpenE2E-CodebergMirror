use crate::backend::managers::storage_manager::{AutosaveWorker, load_from_file};
use crate::backend::managers::user::User;
use serde_json::Value;

// SerializedUser

#[derive(Clone)]
pub struct SerializedUser {
    pub name: String,
    pub data: Value, // Encrypted JSON
}

impl SerializedUser {
    fn new(name: String, data: Value) -> Self {
        Self { name, data }
    }
}

// UserManager

const STORAGE_FILEPATH: &str = "storage.OpenE2E.json";

pub struct UserManager {
    users: Vec<SerializedUser>,
    current_user: Option<User>,
    autosave_worker: AutosaveWorker,
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new(STORAGE_FILEPATH)
    }
}

impl UserManager {
    pub fn new(filepath: &str) -> Self {
        let mut manager = Self {
            users: Vec::new(),
            current_user: None,
            autosave_worker: AutosaveWorker::new(filepath.to_string()),
        };

        // Load existing users from disk
        if let Ok(users_data) = load_from_file(filepath) {
            let _ = manager.import_users(users_data);
        }

        manager
    }

    // User operations

    /// Creates a new user with the given name and password
    pub fn new_user(&mut self, name: &str, password: &str) -> Result<(), String> {
        if self.user_exists(name) {
            return Err(format!("User '{}' already exists", name));
        }

        let user = User::new(name, password)?;
        let serialized_data = user.serialize()?;

        self.users
            .push(SerializedUser::new(name.to_string(), serialized_data));

        Ok(())
    }

    /// Deletes a user by name
    pub fn delete_user(&mut self, name: &str) {
        if self.is_current_user(name) {
            self.current_user = None;
        }

        self.users.retain(|user| user.name != name);
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
        let user = User::deserialize(serialized_user.data.clone(), password)?;
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
    pub fn autosave(&mut self) -> Result<(), String> {
        self.sync_current_user_to_storage()?;

        let data = self.export_users()?;
        self.autosave_worker.queue_save(data)
    }

    /// Syncs the current user's data back to the users list
    fn sync_current_user_to_storage(&mut self) -> Result<(), String> {
        if let Some(ref current_user) = self.current_user {
            let serialized_data = current_user.serialize()?;

            if let Some(stored_user) = self.users.iter_mut().find(|u| u.name == current_user.name) {
                stored_user.data = serialized_data;
            }
        }

        Ok(())
    }

    /// Exports all users to a JSON object
    pub fn export_users(&self) -> Result<Value, String> {
        let mut users_map = serde_json::Map::new();

        for user in &self.users {
            users_map.insert(user.name.clone(), user.data.clone());
        }

        Ok(Value::Object(users_map))
    }

    /// Imports users from a JSON object
    pub fn import_users(&mut self, users_json: Value) -> Result<(), String> {
        let users_map = users_json
            .as_object()
            .ok_or("Expected users JSON to be an object")?;

        self.users.clear();
        self.current_user = None;

        for (username, user_data) in users_map {
            self.users
                .push(SerializedUser::new(username.clone(), user_data.clone()));
        }

        Ok(())
    }

    /// Ensures all data is saved and shud down autosave worker
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.autosave()?;
        self.autosave_worker.shutdown()
    }
}
