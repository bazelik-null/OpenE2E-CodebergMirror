use log::{self, error, info};
use std::io::{self, Write};

use crate::backend::orchestrator::Orchestrator;
use crate::frontend::commands::{Command, scan_commands};

pub struct Application {
    orchestrator: Orchestrator,
    should_exit: bool,
}

impl Application {
    pub fn new() -> Application {
        Application {
            orchestrator: Orchestrator::new(),
            should_exit: false,
        }
    }

    pub fn main_loop(&mut self) {
        loop {
            if self.should_exit {
                break;
            }

            let input = self.get_input();

            if let Err(error) = self.command_dispatch(&input) {
                error!("{}", error)
            }
        }
    }

    fn get_input(&self) -> String {
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

    fn command_dispatch(&mut self, input: &str) -> Result<(), String> {
        // Main command dispatch
        if let Some(command) = scan_commands(input) {
            match command {
                Command::Exit => {
                    info!("Exiting...");
                    self.should_exit = true;
                }
                Command::NewUser => self.user_creation()?,
                Command::DeleteUser => self.user_deletion()?,
                Command::ListUsers => self.users_list(),
                _ => todo!(),
            }
        }
        Ok(())
    }

    fn user_creation(&mut self) -> Result<(), String> {
        println!("Enter username:");

        let name = self.get_input();

        println!("Enter password:");

        let password = self.get_input();

        info!("Creating user...");

        let user = self.orchestrator.create_user(&name, &password)?;

        info!("Created user: {}", user.name);

        Ok(())
    }

    fn users_list(&mut self) {
        println!("Users:");
        let uuids = self.orchestrator.get_users_uuids();
        for uuid in uuids {
            println!("{}", uuid);
        }
    }

    fn user_deletion(&mut self) -> Result<(), String> {
        println!("Enter username:");

        let name = self.get_input();

        self.orchestrator.delete_user(&name);

        info!("Deleted user: {}", name);

        Ok(())
    }
}
