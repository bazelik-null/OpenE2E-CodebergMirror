use crate::backend::managers::user::User;

pub struct UserManager {
    users: Vec<User>,
    current_user: Option<usize>, // Index of selected user, None if no selection
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UserManager {
    pub fn new() -> UserManager {
        UserManager {
            users: vec![],
            current_user: None,
        }
    }

    pub fn new_user(&mut self, name: &str, password: &str) -> Result<&User, String> {
        let user = User::new(name, password)?;

        self.users.push(user);

        Ok(self.users.last().unwrap())
    }

    pub fn delete_user(&mut self, name: &str) {
        let removed_idx = self.users.iter().position(|u| u.name == name);
        self.users.retain(|user| user.name != name);

        // Clear selection if we deleted the current user
        if let Some(idx) = removed_idx {
            if self.current_user == Some(idx) {
                self.current_user = None;
            } else if let Some(curr) = self.current_user {
                // Adjust index if a user before the current was removed
                if idx < curr {
                    self.current_user = Some(curr - 1);
                }
            }
        }
    }

    pub fn get_usernames(&self) -> Vec<&str> {
        self.users.iter().map(|user| user.name.as_str()).collect()
    }

    pub fn select_user(&mut self, name: &str) -> Result<(), String> {
        let current_user = self.users.iter().position(|s| s.name == name);

        match current_user {
            Some(current_user) => self.current_user = Some(current_user),
            None => return Err(format!("User '{}' not found", name)),
        }

        Ok(())
    }

    pub fn deselect_user(&mut self) {
        self.current_user = None;
    }

    pub fn get_current_user(&self) -> Option<&User> {
        self.current_user.and_then(|idx| self.users.get(idx))
    }

    pub fn get_current_user_mut(&mut self) -> Option<&mut User> {
        let idx = self.current_user?;
        self.users.get_mut(idx)
    }

    // TODO
    pub fn deserialize_users() -> Option<Vec<User>> {
        None
    }

    // TODO
    pub fn serialize_users() -> Option<Vec<User>> {
        None
    }
}
