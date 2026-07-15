use colorize::AnsiColor;
use log::{self, error, info};
use std::io::{self, Write};

use crate::backend::orchestrator::Orchestrator;
use crate::frontend::commands::{Command, scan_commands};

pub struct Application {
    orchestrator: Orchestrator,
    should_exit: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Application {
        Application {
            orchestrator: Orchestrator::new(),
            should_exit: false,
        }
    }

    pub fn main_loop(&mut self) {
        self.display_welcome();

        loop {
            if self.should_exit {
                break;
            }

            let input = self.prompt_input();

            if let Err(error) = self.command_dispatch(&input) {
                error!("{}", error);
            }
        }
    }

    // Display Methods

    fn display_welcome(&self) {
        println!();
        println!("{}", "#".repeat(34).cyan());
        println!("{}", "### OpenE2E CLI interface v0.1 ###".cyan());
        println!("{}", "#".repeat(34).cyan());
        println!("{}", "Type 'help' for available commands".cyan());
        println!();
    }

    fn prompt_input(&self) -> String {
        print!("{} ", ">".green());
        if let Err(error) = io::stdout().flush() {
            error!("{}", error)
        }

        let mut input = String::new();
        if let Err(error) = io::stdin().read_line(&mut input) {
            error!("{}", error)
        }

        input.trim().to_string()
    }

    // Command Dispatch

    fn command_dispatch(&mut self, input: &str) -> Result<(), String> {
        if input.is_empty() {
            return Ok(());
        }

        if let Some(command) = scan_commands(input) {
            match command {
                Command::Exit => self.handle_exit(),
                Command::NewUser => self.user_creation()?,
                Command::DeleteUser => self.user_deletion()?,
                Command::ListUsers => self.users_list(),
                Command::Help => self.display_help(),
                _ => info!("Command not yet implemented"),
            }
        }

        Ok(())
    }

    // User Management

    fn user_creation(&mut self) -> Result<(), String> {
        println!();
        println!("{}", "Create New User".yellow().bold());
        println!("{}", "-".repeat(40).black());

        println!("{}", "Username:".black());
        let name = self.prompt_input();

        println!("{}", "Password:".black());
        let password = self.prompt_input();

        info!("Creating user...");
        let user = self.orchestrator.create_user(&name, &password)?;
        info!("User '{}' created successfully", user.name);
        println!();

        Ok(())
    }

    fn user_deletion(&mut self) -> Result<(), String> {
        println!();
        println!("{}", "Delete User".yellow().bold());
        println!("{}", "-".repeat(40).black());

        println!("{}", "Username:".black());
        let name = self.prompt_input();

        self.orchestrator.delete_user(&name);
        info!("User '{}' deleted", name);
        println!();

        Ok(())
    }

    fn users_list(&self) {
        println!();
        println!("{}", "Users List".yellow().bold());
        println!("{}", "-".repeat(40).black());

        let usernames = self.orchestrator.get_usernames();

        if usernames.is_empty() {
            println!("No users found");
        } else {
            for (index, username) in usernames.iter().enumerate() {
                println!("  {}. {}", index + 1, username);
            }
        }
        println!();
    }

    // Main commands

    fn handle_exit(&mut self) {
        info!("Exiting application...");
        self.should_exit = true;
    }

    fn display_help(&self) {
        println!();
        println!("{}", "Available Commands".yellow().bold());
        println!("{}", "-".repeat(40).black());
        println!("  {} - Exit the application", "exit".cyan());
        println!("  {} - Show this help message", "help".cyan());
        println!();
        println!("{}", "User Management".yellow().bold());
        println!("  {} - Create a new user", "u new".cyan());
        println!("  {} - Delete a user", "u delete".cyan());
        println!("  {} - List all users", "u list".cyan());
        println!();
    }
}
