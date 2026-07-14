use uuid::Uuid;

use crate::backend::managers::user_manager::User;

static KEY_COUNT: usize = 1;

pub struct Orchestrator {
    users: Vec<User>,
}

impl Orchestrator {
    pub fn new() -> Orchestrator {
        Orchestrator { users: vec![] }
    }

    pub fn create_user(&mut self, password: &str) -> Result<&User, String> {
        let user = User::new(KEY_COUNT, password)?;

        self.users.push(user);

        Ok(self.users.last().unwrap())
    }

    pub fn delete_user(&mut self, uid: Uuid) {
        self.users.retain(|user| user.uid != uid);
    }

    pub fn get_users_uuids(&self) -> Vec<Uuid> {
        self.users.iter().map(|user| user.uid).collect()
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
