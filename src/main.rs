/*
 * Copyright (C) 2026 bazelik-dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use log::error;

pub mod backend;
pub mod error_mapper;
pub mod frontend;

// Default build: interactive CLI frontend.
#[cfg(not(feature = "gui"))]
fn main() {
    use crate::frontend::{cli::Application, logger};

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

// `--features gui`: Slint desktop/mobile frontend.
#[cfg(feature = "gui")]
fn main() {
    use crate::frontend::logger;

    logger::init().unwrap();

    if let Err(error) = frontend::gui::run() {
        error!("{}", error);
    }
}
