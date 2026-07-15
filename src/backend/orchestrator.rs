use crate::backend::managers::user_manager::User;

static KEY_COUNT: usize = 1;

pub struct Orchestrator {
    users: Vec<User>,
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Orchestrator {
    pub fn new() -> Orchestrator {
        Orchestrator { users: vec![] }
    }

    pub fn create_user(&mut self, name: &str, password: &str) -> Result<&User, String> {
        let user = User::new(KEY_COUNT, name, password)?;

        self.users.push(user);

        Ok(self.users.last().unwrap())
    }

    pub fn delete_user(&mut self, name: &str) {
        self.users.retain(|user| user.name != name);
    }

    pub fn get_usernames(&self) -> Vec<&str> {
        self.users.iter().map(|user| user.name.as_str()).collect()
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
