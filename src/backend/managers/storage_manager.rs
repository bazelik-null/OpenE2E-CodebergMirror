use fjall::{Database, Keyspace, KeyspaceCreateOptions};
use log::{debug, error, info};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::error_mapper::MapErrorToString;

/// Manages all autosave operations and DB
pub struct DatabaseManager {
    db: Arc<Database>,
    pub shutdown_pair: (Arc<Mutex<bool>>, Condvar),
}

impl DatabaseManager {
    /// Creates a new manager and initializes the database
    pub fn new(db_path: &str) -> Result<Self, String> {
        let db = Self::initialize_database(db_path)?;
        info!("AutosaveWorker initialized with database at: {}", db_path);

        Ok(DatabaseManager {
            db: Arc::new(db),
            shutdown_pair: (Arc::new(Mutex::new(false)), Condvar::new()),
        })
    }

    /// Initializes the database with required keyspaces
    fn initialize_database(db_path: &str) -> Result<Database, String> {
        let db = Database::builder(db_path).open().map_err_to_string()?;

        // Create keyspaces
        let _ = db
            .keyspace("users", KeyspaceCreateOptions::default)
            .map_err_to_string()?;
        let _ = db
            .keyspace("sessions", KeyspaceCreateOptions::default)
            .map_err_to_string()?;
        let _ = db
            .keyspace("messages", KeyspaceCreateOptions::default)
            .map_err_to_string()?;
        let _ = db
            .keyspace("user_sessions", KeyspaceCreateOptions::default)
            .map_err_to_string()?;
        let _ = db
            .keyspace("session_messages", KeyspaceCreateOptions::default)
            .map_err_to_string()?;

        Ok(db)
    }

    // Generic operations

    /// Generic save operation for any entity
    fn save_entity(
        &self,
        keyspace_name: &str,
        key: &str,
        data: &[u8],
        entity_type: &str,
        context: &str,
    ) -> Result<(), String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        keyspace.insert(key.as_bytes(), data).map_err_to_string()?;
        debug!("Saved {}: {} ({} bytes)", entity_type, context, data.len());
        Ok(())
    }

    /// Generic get operation for any entity
    fn get_entity(
        &self,
        keyspace_name: &str,
        key: &str,
        entity_type: &str,
    ) -> Result<Vec<u8>, String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        keyspace
            .get(key.as_bytes())
            .map_err_to_string()?
            .ok_or_else(|| format!("Couldn't find {}: {}", entity_type, key))
            .map(|v| v.to_vec())
    }

    /// Generic delete operation for any entity
    fn delete_entity(
        &self,
        keyspace_name: &str,
        key: &str,
        entity_type: &str,
    ) -> Result<(), String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        keyspace.remove(key.as_bytes()).map_err_to_string()?;
        debug!("Deleted {}: {}", entity_type, key);
        Ok(())
    }

    // User operations

    pub fn save_user(&self, user_id: &str, data: &[u8]) -> Result<(), String> {
        self.save_entity("users", user_id, data, "user", user_id)
    }

    pub fn get_user(&self, user_id: &str) -> Result<Vec<u8>, String> {
        self.get_entity("users", user_id, "user")
    }

    pub fn get_all_users(&self) -> Result<Vec<(String, Vec<u8>)>, String> {
        self.get_all_from_keyspace("users")
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), String> {
        self.delete_entity("users", user_id, "user")
    }

    // Session operations

    pub fn save_session(&self, session_id: &str, user_id: &str, data: &[u8]) -> Result<(), String> {
        self.save_entity(
            "sessions",
            session_id,
            data,
            "session",
            &format!("{} for user: {}", session_id, user_id),
        )?;
        self.add_to_index("user_sessions", user_id, session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Vec<u8>, String> {
        self.get_entity("sessions", session_id, "session")
    }

    pub fn get_sessions_by_user(&self, user_id: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        self.get_related_entities("user_sessions", user_id, "sessions", "sessions")
    }

    pub fn delete_session(&self, session_id: &str, user_id: &str) -> Result<(), String> {
        self.delete_entity("sessions", session_id, "session")?;
        self.remove_from_index("user_sessions", user_id, session_id)
    }

    // Messages operations

    pub fn save_message(
        &self,
        message_id: &str,
        session_id: &str,
        data: &[u8],
    ) -> Result<(), String> {
        self.save_entity(
            "messages",
            message_id,
            data,
            "message",
            &format!("{} for session: {}", message_id, session_id),
        )?;
        self.add_to_index("session_messages", session_id, message_id)
    }

    pub fn get_message(&self, message_id: &str) -> Result<Vec<u8>, String> {
        self.get_entity("messages", message_id, "message")
    }

    pub fn get_messages_by_session(&self, session_id: &str) -> Result<Vec<Vec<u8>>, String> {
        let keyspace = self.get_keyspace("session_messages")?;
        let index_key = format!("session:{}:messages", session_id);

        match keyspace.get(index_key.as_bytes()).map_err_to_string()? {
            Some(data) => {
                let message_ids: Vec<String> = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize message list: {}", e))?;
                self.fetch_entities("messages", &message_ids)
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn get_message_ids_by_session(&self, session_id: &str) -> Result<Vec<String>, String> {
        let keyspace = self.get_keyspace("session_messages")?;
        let index_key = format!("session:{}:messages", session_id);

        match keyspace.get(index_key.as_bytes()).map_err_to_string()? {
            Some(data) => {
                let message_ids: Vec<String> = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize message list: {}", e))?;
                Ok(message_ids)
            }
            None => Ok(Vec::new()),
        }
    }

    pub fn delete_message(&self, message_id: &str, session_id: &str) -> Result<(), String> {
        self.delete_entity("messages", message_id, "message")?;
        self.remove_from_index("session_messages", session_id, message_id)
    }

    // Index management

    /// Generic operation to add an ID to an index
    fn add_to_index(
        &self,
        index_keyspace: &str,
        parent_id: &str,
        child_id: &str,
    ) -> Result<(), String> {
        let keyspace = self.get_keyspace(index_keyspace)?;
        let index_key = self.build_index_key(index_keyspace, parent_id);

        let mut ids = self.load_index(&keyspace, &index_key)?;

        if !ids.contains(&child_id.to_string()) {
            ids.push(child_id.to_string());
        }

        self.save_index(&keyspace, &index_key, &ids)
    }

    /// Generic operation to remove an ID from an index
    fn remove_from_index(
        &self,
        index_keyspace: &str,
        parent_id: &str,
        child_id: &str,
    ) -> Result<(), String> {
        let keyspace = self.get_keyspace(index_keyspace)?;
        let index_key = self.build_index_key(index_keyspace, parent_id);

        if let Some(data) = keyspace.get(index_key.as_bytes()).map_err_to_string()? {
            let mut ids: Vec<String> = serde_json::from_slice(&data).unwrap_or_default();
            ids.retain(|id| id != child_id);

            if ids.is_empty() {
                keyspace.remove(index_key.as_bytes()).map_err_to_string()?;
            } else {
                self.save_index(&keyspace, &index_key, &ids)?;
            }
        }

        Ok(())
    }

    /// Builds standardized index keys
    fn build_index_key(&self, index_keyspace: &str, parent_id: &str) -> String {
        if index_keyspace.contains("user_sessions") {
            format!("user:{}:sessions", parent_id)
        } else if index_keyspace.contains("session_messages") {
            format!("session:{}:messages", parent_id)
        } else {
            format!("{}:{}:index", index_keyspace, parent_id)
        }
    }

    /// Loads an index from the database
    fn load_index(&self, keyspace: &Keyspace, key: &str) -> Result<Vec<String>, String> {
        match keyspace.get(key.as_bytes()).map_err_to_string()? {
            Some(data) => serde_json::from_slice::<Vec<String>>(&data)
                .map_err(|e| format!("Failed to deserialize index: {}", e)),
            None => Ok(Vec::new()),
        }
    }

    /// Saves an index to the database
    fn save_index(&self, keyspace: &Keyspace, key: &str, ids: &[String]) -> Result<(), String> {
        let serialized =
            serde_json::to_vec(ids).map_err(|e| format!("Failed to serialize index: {}", e))?;
        keyspace
            .insert(key.as_bytes(), &serialized)
            .map_err_to_string()
    }

    // Retrieval

    /// Generic method to get all entries from a keyspace
    fn get_all_from_keyspace(&self, keyspace_name: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        let mut entries = Vec::new();

        for result in keyspace.iter() {
            let (key, value) = result.into_inner().map_err_to_string()?;

            let key_str = String::from_utf8(key.to_vec())
                .map_err(|e| format!("Failed to convert key to string: {}", e))?;
            entries.push((key_str, value.to_vec()));
        }

        Ok(entries)
    }

    /// Fetch multiple entities by their IDs
    fn fetch_entities(&self, keyspace_name: &str, ids: &[String]) -> Result<Vec<Vec<u8>>, String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        let mut entities = Vec::new();

        for id in ids {
            match keyspace.get(id.as_bytes()).map_err_to_string() {
                Ok(Some(data)) => entities.push(data.to_vec()),
                Ok(None) => debug!("Entity not found: {}", id),
                Err(e) => error!("Failed to retrieve entity {}: {}", id, e),
            }
        }

        Ok(entities)
    }

    /// Get related entities through an index
    fn get_related_entities(
        &self,
        index_keyspace: &str,
        parent_id: &str,
        entity_keyspace: &str,
        entity_type: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, String> {
        let index_ks = self.get_keyspace(index_keyspace)?;
        let index_key = self.build_index_key(index_keyspace, parent_id);

        match index_ks.get(index_key.as_bytes()).map_err_to_string()? {
            Some(data) => {
                let ids: Vec<String> = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize index: {}", e))?;

                let entities_ks = self.get_keyspace(entity_keyspace)?;
                let mut entities = Vec::new();

                for id in ids {
                    match entities_ks.get(id.as_bytes()).map_err_to_string() {
                        Ok(Some(data)) => entities.push((id, data.to_vec())),
                        Ok(None) => debug!("{} not found: {}", entity_type, id),
                        Err(e) => error!("Failed to retrieve {} {}: {}", entity_type, id, e),
                    }
                }

                Ok(entities)
            }
            None => Ok(Vec::new()),
        }
    }

    // Utility functions

    pub fn list_keys(&self, keyspace_name: &str) -> Result<Vec<Vec<u8>>, String> {
        let keyspace = self.get_keyspace(keyspace_name)?;
        let mut keys = Vec::new();

        for result in keyspace.iter() {
            keys.push(result.key().map_err_to_string()?.to_vec());
        }

        Ok(keys)
    }

    fn get_keyspace(&self, name: &str) -> Result<Keyspace, String> {
        self.db
            .keyspace(name, KeyspaceCreateOptions::default)
            .map_err_to_string()
    }

    /// Initiates graceful shutdown of the autosave worker
    pub fn shutdown(&self) -> Result<(), String> {
        let mut flag = self.shutdown_pair.0.lock().map_err(|e| e.to_string())?;
        *flag = true;

        self.shutdown_pair.1.notify_all();
        info!("Autosave worker shutdown initiated");

        Ok(())
    }

    /// Saves DB to disk
    pub fn flush(&self) -> Result<(), String> {
        self.db
            .persist(fjall::PersistMode::SyncAll)
            .map_err_to_string()?;
        debug!("Database saved successfully");
        Ok(())
    }
}

/// Background worker that performs periodic autosaves
pub struct BackgroundWorker {
    manager: Arc<DatabaseManager>,
    autosave_interval: Duration,
}

impl BackgroundWorker {
    /// Creates a new background worker with the specified autosave interval
    pub fn new(autosave_interval: Duration, db_path: &str) -> Result<Self, String> {
        let worker = DatabaseManager::new(db_path)?;
        Ok(BackgroundWorker {
            manager: Arc::new(worker),
            autosave_interval,
        })
    }

    /// Starts the background worker thread
    /// The worker will periodically flush the database and perform maintenance until shutdown is requested via the returned handle
    pub fn start(self) -> WorkerHandle {
        let worker = Arc::clone(&self.manager);
        let interval = self.autosave_interval;

        let handle = thread::spawn(move || {
            loop {
                let (shutdown_flag, condvar) = &worker.shutdown_pair;
                let mut flag = shutdown_flag.lock().unwrap();

                let result = condvar.wait_timeout(flag, interval).unwrap();
                flag = result.0;

                if *flag {
                    info!("Background worker received shutdown signal");
                    break;
                }

                drop(flag);

                if let Err(e) = worker.flush() {
                    error!("Periodic flush failed: {}", e);
                } else {
                    debug!("Periodic flush completed");
                }
            }

            debug!("Background worker thread terminated");
        });

        WorkerHandle {
            worker: Arc::clone(&self.manager),
            thread_handle: handle,
        }
    }
}

/// Handle for managing the background worker
pub struct WorkerHandle {
    worker: Arc<DatabaseManager>,
    thread_handle: JoinHandle<()>,
}

impl WorkerHandle {
    /// Requests shutdown and waits for the worker to terminate
    pub fn shutdown(self) -> Result<(), String> {
        self.worker.shutdown()?;
        self.thread_handle
            .join()
            .map_err(|_| "Failed to join worker thread".to_string())?;
        Ok(())
    }

    /// Performs final flush and compact before shutdown
    pub fn graceful_shutdown(self) -> Result<(), String> {
        self.worker.flush()?;
        self.shutdown()
    }

    /// Returns a reference to the autosave worker
    pub fn worker(&self) -> &Arc<DatabaseManager> {
        &self.worker
    }
}
