use log::error;
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

enum SaveMessage {
    Save(Value),
    Shutdown,
}

// Autosave Worker

/// Manages background saving of data to disk
/// The worker defers writes to disk, adding multiple save requests into a single write operation
pub struct AutosaveWorker {
    sender: mpsc::Sender<SaveMessage>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl AutosaveWorker {
    const DEBOUNCE_DURATION_MS: u64 = 500;

    /// Creates a new autosave worker for the given filepath
    /// Spawns a background thread that handles all disk I/O operations
    pub fn new(filepath: String) -> Self {
        let (sender, receiver) = mpsc::channel();

        let thread_handle = thread::spawn(move || {
            Self::worker_loop(receiver, filepath);
        });

        Self {
            sender,
            thread_handle: Some(thread_handle),
        }
    }

    // Save Operations

    /// Queues data to be saved to disk
    pub fn queue_save(&self, data: Value) -> Result<(), String> {
        self.sender
            .send(SaveMessage::Save(data))
            .map_err(|_| "Autosave worker disconnected".to_string())
    }

    /// Ensures final data is saved
    pub fn shutdown(&mut self) -> Result<(), String> {
        let _ = self.sender.send(SaveMessage::Shutdown);

        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| "Failed to join autosave worker thread".to_string())
        } else {
            Ok(())
        }
    }

    // Worker Loop

    /// Main loop running in the background thread
    /// Processes save requests with debouncing to add multiple writes
    fn worker_loop(receiver: mpsc::Receiver<SaveMessage>, filepath: String) {
        let debounce_duration = Duration::from_millis(Self::DEBOUNCE_DURATION_MS);
        let mut pending_data: Option<Value> = None;
        let mut debounce_timer: Option<Instant> = None;

        loop {
            match receiver.recv_timeout(debounce_duration) {
                Ok(SaveMessage::Save(data)) => {
                    pending_data = Some(data);
                    debounce_timer = Some(Instant::now());
                }
                Ok(SaveMessage::Shutdown) => {
                    Self::perform_final_save(&filepath, pending_data);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    Self::check_debounce_and_save(
                        &filepath,
                        &mut pending_data,
                        debounce_timer,
                        debounce_duration,
                    );
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    }

    /// Checks if the debounce timer has elapsed and performs a save if needed
    fn check_debounce_and_save(
        filepath: &str,
        pending_data: &mut Option<Value>,
        debounce_timer: Option<Instant>,
        debounce_duration: Duration,
    ) {
        let should_save = debounce_timer
            .map(|timer| timer.elapsed() >= debounce_duration)
            .unwrap_or(false);

        if !should_save {
            return;
        }

        if let Some(data) = pending_data.take()
            && let Err(e) = Self::write_to_disk(filepath, &data)
        {
            error!("Autosave failed: {}", e);
            // Re-queue data for retry
            *pending_data = Some(data);
        }
    }

    /// Performs a final save before shutdown
    fn perform_final_save(filepath: &str, pending_data: Option<Value>) {
        if let Some(data) = pending_data
            && let Err(e) = Self::write_to_disk(filepath, &data)
        {
            error!("Final autosave before shutdown failed: {}", e);
        }
    }

    // Disk I/O

    /// Writes data to disk as formatted JSON.
    fn write_to_disk(filepath: &str, data: &Value) -> Result<(), String> {
        std::fs::write(filepath, data.to_string())
            .map_err(|e| format!("Failed to write to disk: {}", e))
    }
}

impl Drop for AutosaveWorker {
    fn drop(&mut self) {
        // Attempt graceful shutdown, but ignore errors
        let _ = self.sender.send(SaveMessage::Shutdown);
    }
}

// Storage Utilities

/// Loads JSON data from a file
/// Returns an empty JSON object if the file doesn't exist yet
pub fn load_from_file(filepath: &str) -> Result<Value, String> {
    match std::fs::read_to_string(filepath) {
        Ok(content) => parse_json_content(&content),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet; return empty object
            Ok(Value::Object(serde_json::Map::new()))
        }
        Err(e) => Err(format!("Failed to read file '{}': {}", filepath, e)),
    }
}

/// Parses JSON content from a string
fn parse_json_content(content: &str) -> Result<Value, String> {
    serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {}", e))
}
