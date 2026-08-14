/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::time::Duration;

use crate::backend::managers::storage_manager::{
    BackgroundWorker, WorkerHandle, get_storage_filepath,
};

const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(60);

pub struct Repository {
    pub db_handle: WorkerHandle,
}

impl Repository {
    /// Creates new DB or opens existing at standard storage filepath
    pub fn new() -> Result<Self, String> {
        let storage_path = get_storage_filepath();
        let storage_path = storage_path.to_string_lossy().into_owned();
        Self::with_storage_path(&storage_path)
    }

    /// Creates new DB or opens existing at specified storage filepath
    pub fn with_storage_path(db_path: &str) -> Result<Self, String> {
        let worker = BackgroundWorker::new(AUTOSAVE_INTERVAL, db_path)?;
        let handle = worker.start();

        Ok(Self { db_handle: handle })
    }

    /// Gracefully shuts down the database worker
    pub fn shutdown(self) -> Result<(), String> {
        self.db_handle.graceful_shutdown()
    }
}
