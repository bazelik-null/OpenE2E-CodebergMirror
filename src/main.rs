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
