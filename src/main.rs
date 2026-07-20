use crate::frontend::{cli::Application, logger};

pub mod backend;
pub mod error_mapper;
pub mod frontend;

fn main() {
    // Set up logger
    logger::init().unwrap();

    // Create application instance
    let mut app = Application::new();

    // Launch main loop
    app.main_loop();
}
