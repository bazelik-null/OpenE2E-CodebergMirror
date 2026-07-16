use log::error;
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

enum SaveMessage {
    Save(Value),
    Shutdown,
}

pub struct AutosaveWorker {
    tx: mpsc::Sender<SaveMessage>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl AutosaveWorker {
    pub fn new(filepath: String) -> Self {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            Self::worker_loop(rx, filepath);
        });

        Self {
            tx,
            thread_handle: Some(handle),
        }
    }

    fn worker_loop(rx: mpsc::Receiver<SaveMessage>, filepath: String) {
        let mut pending_data: Option<Value> = None;
        let mut debounce_timer: Option<Instant> = None;
        let debounce_duration = Duration::from_millis(500);

        loop {
            let timeout = debounce_duration;

            match rx.recv_timeout(timeout) {
                Ok(SaveMessage::Save(data)) => {
                    pending_data = Some(data);
                    debounce_timer = Some(Instant::now());
                }
                Ok(SaveMessage::Shutdown) => {
                    // Final save before shutdown
                    if let Some(data) = pending_data {
                        let _ = Self::write_to_disk(&filepath, &data);
                    }
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(timer) = debounce_timer
                        && timer.elapsed() >= debounce_duration
                    {
                        if let Some(data) = pending_data.take()
                            && let Err(error) = Self::write_to_disk(&filepath, &data)
                        {
                            error!("Autosave failed: {}", error);
                            // Re-queue for retry
                            pending_data = Some(data);
                        }
                        debounce_timer = None;
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    }

    fn write_to_disk(filepath: &str, data: &Value) -> Result<(), String> {
        std::fs::write(filepath, data.to_string()).map_err(|e| format!("IO error: {}", e))
    }

    pub fn queue_save(&self, data: Value) -> Result<(), String> {
        self.tx
            .send(SaveMessage::Save(data))
            .map_err(|_| "Autosave worker disconnected".to_string())
    }

    pub fn shutdown(&mut self) -> Result<(), String> {
        let _ = self.tx.send(SaveMessage::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| "Failed to join autosave worker".to_string())
        } else {
            Ok(())
        }
    }
}

impl Drop for AutosaveWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(SaveMessage::Shutdown);
    }
}

pub fn load_from_file(filepath: &str) -> Result<Value, String> {
    match std::fs::read_to_string(filepath) {
        Ok(content) => serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON from savefile: {}", e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet
            Ok(Value::Object(serde_json::Map::new()))
        }
        Err(e) => Err(format!("Failed to read savefile: {}", e)),
    }
}
