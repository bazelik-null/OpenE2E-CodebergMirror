use log::{self, error, info};
use std::io::{self, Write};

use crate::frontend::commands::{Command, scan_commands};

pub fn main_loop() {
    loop {
        let input = get_input();

        // Main command dispatch
        if let Some(command) = scan_commands(input.as_str()) {
            match command {
                Command::Exit => {
                    info!("Exiting...");
                    break;
                }
                Command::NewSession => todo!(),
                Command::ExitSession => todo!(),
                Command::OpenSession => todo!(),
                Command::Encrypt(_) => todo!(),
                Command::Decrypt(_) => todo!(),
            }
        }
    }
}

fn get_input() -> String {
    let mut input = String::new();

    // Display prompt
    print!("> ");
    if let Err(error) = io::stdout().flush() {
        error!("{}", error)
    }

    // Read input
    if let Err(error) = io::stdin().read_line(&mut input) {
        error!("{}", error)
    }
    input.trim().to_string()
}
