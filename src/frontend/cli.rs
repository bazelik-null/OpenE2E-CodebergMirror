use colorize::AnsiColor;
use log::{error, info};
use std::io::{self, Write};

use crate::backend::managers::user::User;
use crate::backend::managers::user_manager::UserManager;
use crate::frontend::commands::{Command, scan_commands};

const HEADER_WIDTH: usize = 34;
const SECTION_WIDTH: usize = 40;

pub struct Application {
    user_manager: UserManager,
    should_exit: bool,
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

impl Application {
    pub fn new() -> Self {
        Self {
            user_manager: UserManager::new(),
            should_exit: false,
        }
    }

    pub fn main_loop(&mut self) {
        self.display_welcome();

        while !self.should_exit {
            let input = prompt_input();

            if let Err(err) = self.command_dispatch(&input) {
                error!("{}", err);
            }
        }
    }

    fn display_welcome(&self) {
        println!();
        println!("{}", "#".repeat(HEADER_WIDTH).cyan());
        println!("{}", "### OpenE2E CLI interface v0.3 ###".cyan());
        println!("{}", "#".repeat(HEADER_WIDTH).cyan());
        println!("{}", "Type 'help' for available commands".cyan());
        println!();
    }

    fn command_dispatch(&mut self, input: &str) -> Result<(), String> {
        if input.is_empty() {
            return Ok(());
        }

        let command =
            scan_commands(input).ok_or("Unknown command. Type 'help' for available commands.")?;

        match command {
            Command::Exit => self.handle_exit(),
            Command::Help => self.display_help(),
            Command::Encrypt { text } => self.encrypt(&text)?,
            Command::Decrypt { text } => self.decrypt(&text)?,
            Command::NewUser { name, password } => self.user_creation(&name, &password)?,
            Command::DeleteUser { name } => self.user_deletion(&name)?,
            Command::ListUsers => self.users_list(),
            Command::LoginUser { name } => self.user_login(&name)?,
            Command::LogoutUser => self.user_logout()?,
            Command::NewSession { name } => self.session_creation(&name)?,
            Command::DeleteSession { name } => self.session_deletion(&name)?,
            Command::ListSessions => self.session_list()?,
            Command::OpenSession { name } => self.session_open(&name)?,
            Command::ExitSession => self.session_close()?,
        }

        Ok(())
    }

    fn user_creation(&mut self, name: &str, password: &str) -> Result<(), String> {
        info!("Creating user...");
        let user = self.user_manager.new_user(name, password)?;
        info!("User '{}' created successfully", user.name);

        self.user_manager.select_user(name)?;
        println!();

        Ok(())
    }

    fn user_deletion(&mut self, name: &str) -> Result<(), String> {
        self.user_manager.delete_user(name);
        info!("User '{}' deleted", name);
        println!();
        Ok(())
    }

    fn users_list(&self) {
        self.display_section("Users List");
        let usernames = self.user_manager.get_usernames();

        if usernames.is_empty() {
            println!("No users found");
        } else {
            usernames
                .iter()
                .enumerate()
                .for_each(|(i, name)| println!("  {}. {}", i + 1, name));
        }
        println!();
    }

    fn user_login(&mut self, name: &str) -> Result<(), String> {
        self.user_manager.select_user(name)?;
        info!("User '{}' selected", name);
        println!();
        Ok(())
    }

    fn user_logout(&mut self) -> Result<(), String> {
        self.user_manager.deselect_user();
        info!("Logged out");
        println!();
        Ok(())
    }

    fn session_creation(&mut self, name: &str) -> Result<(), String> {
        self.display_section("Create New Session");

        let session_type = self.prompt_session_type()?;

        info!("Creating session...");
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        match session_type.as_str() {
            "in" => create_inbound_session(user, name)?,
            "out" => create_outbound_session(user, name)?,
            _ => return Err("Unknown session type".to_string()),
        }

        info!("Session '{}' created successfully", name);
        println!();

        Ok(())
    }

    fn session_deletion(&mut self, name: &str) -> Result<(), String> {
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        user.session_manager.delete_session(name);
        info!("Session '{}' deleted", name);
        println!();

        Ok(())
    }

    fn session_list(&mut self) -> Result<(), String> {
        self.display_section("Sessions List");

        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;
        let session_names = user.session_manager.get_session_names();

        if session_names.is_empty() {
            println!("No sessions found");
        } else {
            session_names
                .iter()
                .enumerate()
                .for_each(|(i, name)| println!("  {}. {}", i + 1, name));
        }
        println!();

        Ok(())
    }

    fn session_open(&mut self, name: &str) -> Result<(), String> {
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        user.session_manager.select_session(name)?;
        info!("Session '{}' selected", name);
        println!();

        Ok(())
    }

    fn session_close(&mut self) -> Result<(), String> {
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        user.session_manager.deselect_session();
        info!("Session closed");
        println!();

        Ok(())
    }

    fn encrypt(&mut self, text: &str) -> Result<(), String> {
        println!();

        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        let encrypted = user.session_manager.encrypt(text)?;
        println!(">> {}", encrypted);

        println!();

        Ok(())
    }

    fn decrypt(&mut self, text: &str) -> Result<(), String> {
        println!();

        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or("No user selected")?;

        let encrypted = user.session_manager.decrypt(text)?;
        println!(">> {}", encrypted);

        println!();

        Ok(())
    }

    fn handle_exit(&mut self) {
        info!("Exiting application...");
        self.should_exit = true;
    }

    fn display_help(&self) {
        println!();
        println!("{}", "Available Commands".yellow().bold());
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
        println!("  {} - Exit the application", "exit".cyan());
        println!("  {} - Show this help message", "help".cyan());
        println!();
        println!("{}", "User Management".yellow().bold());
        println!(
            "  {} - Create a new user",
            "u new <username> <password>".cyan()
        );
        println!("  {} - Delete a user", "u delete <username>".cyan());
        println!("  {} - List all users", "u list".cyan());
        println!("  {} - Login", "u login <username>".cyan());
        println!("  {} - Logout", "u logout".cyan());
        println!();
        println!("{}", "Session Management".yellow().bold());
        println!("  {} - Create a new session", "s new <session_name>".cyan());
        println!("  {} - Delete a session", "s delete <session_name>".cyan());
        println!("  {} - List all sessions", "s list".cyan());
        println!("  {} - Open session", "s open <session_name>".cyan());
        println!("  {} - Close session", "s exit".cyan());
        println!();
        println!("{}", "Chatting".yellow().bold());
        println!("  {} - Encrypt text", "e <text>".cyan());
        println!("  {} - Decrypt text", "d <text>".cyan());
        println!();
    }

    fn display_section(&self, title: &str) {
        println!();
        println!("{}", title.to_string().yellow().bold());
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
    }

    fn prompt_session_type(&self) -> Result<String, String> {
        println!("{}", "Session type:".grey());
        println!("  {} (inbound)", "in".green());
        println!("  {} (outbound)", "out".green());
        Ok(prompt_input())
    }
}

fn prompt_input() -> String {
    print!("{} ", ">".green());
    let _ = io::stdout().flush();

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);

    input.trim().to_string()
}

fn create_inbound_session(user: &mut User, name: &str) -> Result<(), String> {
    println!("{}", "Generating your keys...".grey());
    let our_keys = user.session_manager.generate_keys(&mut user.account)?;
    println!("{}", "Share this with the other party:".grey());
    println!("{}", our_keys.bold());
    println!();

    println!("{}", "Paste the other party's keys:".grey());
    let remote_keys = prompt_input();
    println!();

    println!("{}", "Paste init message from them:".grey());
    let first_message = prompt_input();
    println!();

    user.session_manager.establish_session_from_message(
        &mut user.account,
        name,
        &remote_keys,
        &first_message,
    )?;

    user.session_manager.select_session(name)
}

fn create_outbound_session(user: &mut User, name: &str) -> Result<(), String> {
    println!("{}", "Generating your keys...".grey());
    let our_keys = user.session_manager.generate_keys(&mut user.account)?;
    println!("{}", "Share this with the other party:".grey());
    println!("{}", our_keys.bold());
    println!();

    println!("{}", "Paste the other party's keys:".grey());
    let remote_keys = prompt_input();
    println!();

    user.session_manager
        .establish_session(&mut user.account, name, &remote_keys)?;
    user.session_manager.select_session(name)?;

    println!(
        "{}",
        "Session established! Send init message to finish:".green()
    );

    let init_message = user.session_manager.encrypt("")?;
    println!("{}", init_message.bold());

    Ok(())
}
