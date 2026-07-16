use crate::backend::managers::storage_manager::{AutosaveWorker, load_from_file};
use crate::backend::managers::user::User;
use serde_json::Value;

#[derive(Clone)]
pub struct SerializedUser {
    pub name: String,
    pub data: Value, // Encrypted JSON
}

pub struct UserManager {
    users: Vec<SerializedUser>,
    current_user: Option<User>,
    autosave: AutosaveWorker,
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new("storage.OpenE2E.json")
    }
}

impl UserManager {
    pub fn new(filepath: &str) -> Self {
        let mut manager = UserManager {
            users: vec![],
            current_user: None,
            autosave: AutosaveWorker::new(filepath.to_string()),
        };

        // Load existing users from disk
        if let Ok(users_data) = load_from_file(filepath) {
            let _ = manager.import_users(users_data);
        }

        manager
    }

    /// Saves all users and sessions using background thread
    pub fn autosave(&mut self) -> Result<(), String> {
        // Sync current user to users list before exporting
        if let Some(ref current_user) = self.current_user {
            let serialized_data = current_user.serialize()?;

            // Update the stored user data
            if let Some(stored_user) = self.users.iter_mut().find(|u| u.name == current_user.name) {
                stored_user.data = serialized_data;
            }
        }

        let data = self.export_users()?;
        self.autosave.queue_save(data)
    }

    pub fn new_user(&mut self, name: &str, password: &str) -> Result<(), String> {
        // Check if user already exists
        if self.users.iter().any(|u| u.name == name) {
            return Err(format!("User '{}' already exists", name));
        }

        let user = User::new(name, password)?;
        let serialized_data = user.serialize()?;

        self.users.push(SerializedUser {
            name: name.to_string(),
            data: serialized_data,
        });

        Ok(())
    }

    pub fn delete_user(&mut self, name: &str) {
        // Logout if deleted user is current
        if let Some(ref current) = self.current_user
            && current.name == name
        {
            self.current_user = None;
        }

        self.users.retain(|user| user.name != name);
    }

    pub fn get_usernames(&self) -> Vec<&str> {
        self.users.iter().map(|user| user.name.as_str()).collect()
    }

    pub fn login(&mut self, name: &str, password: &str) -> Result<(), String> {
        let serialized_user = self
            .users
            .iter()
            .find(|u| u.name == name)
            .ok_or(format!("User '{}' not found", name))?;

        // Deserialize with password verification
        let user = User::deserialize(serialized_user.data.clone(), password)?;
        self.current_user = Some(user);
        Ok(())
    }

    pub fn logout(&mut self) {
        self.current_user = None;
    }

    pub fn get_current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }

    pub fn get_current_user_mut(&mut self) -> Option<&mut User> {
        self.current_user.as_mut()
    }

    /// Exports all users to JSON
    pub fn export_users(&self) -> Result<Value, String> {
        let mut users_map = serde_json::Map::new();

        for user in &self.users {
            users_map.insert(user.name.clone(), user.data.clone());
        }

        Ok(Value::Object(users_map))
    }

    /// Imports all users from JSON
    pub fn import_users(&mut self, users_json: Value) -> Result<(), String> {
        let users_map = users_json
            .as_object()
            .ok_or("Expected users JSON to be an object")?;

        self.users.clear();
        self.current_user = None;

        for (username, user_data) in users_map {
            self.users.push(SerializedUser {
                name: username.clone(),
                data: user_data.clone(),
            });
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        self.autosave()?; // Final save
        self.autosave.shutdown()
    }
}
