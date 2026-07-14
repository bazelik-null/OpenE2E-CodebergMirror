use crate::frontend::logger;

pub mod backend;
pub mod data;
pub mod frontend;

fn main() {
    logger::init().unwrap();
    frontend::cli::main_loop();
}
