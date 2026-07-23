/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use log::error;

use crate::frontend::{cli::Application, logger};

pub mod backend;
pub mod error_mapper;
pub mod frontend;

fn main() {
    // Set up logger
    logger::init().unwrap();

    // Create application instance and launch main loop
    match Application::new() {
        Ok(mut app) => {
            app.main_loop();

            if let Err(error) = app.shutdown() {
                error!("{}", error);
            }
        }
        Err(error) => error!("{}", error),
    }
}
