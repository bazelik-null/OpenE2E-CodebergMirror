use colorize::AnsiColor;
use log::{error, info};
use std::io::{self, Write};

use crate::backend::managers::user_manager::UserManager;
use crate::backend::objects::user::User;
use crate::frontend::commands::{Command, scan_commands};
use crate::frontend::localization::{Localization, fluent_args};

const HEADER_WIDTH: usize = 34;
const SECTION_WIDTH: usize = 40;
const VERSION: &str = "v0.7";

pub struct Application {
    user_manager: UserManager,
    should_exit: bool,
    localization: Localization,
}

impl Application {
    pub fn new() -> Result<Self, String> {
        let localization = Localization::new("en")
            .map_err(|e| format!("Failed to initialize localization: {}", e))?;

        Ok(Self {
            user_manager: UserManager::new()?,
            should_exit: false,
            localization,
        })
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
        println!(
            "{0} {1} {2} {0}",
            "#".repeat(3).cyan(),
            self.localization.get("welcome-header").cyan(),
            VERSION.cyan()
        );
        println!("{}", "#".repeat(HEADER_WIDTH).cyan());
        println!("{}", self.localization.get("help-tip").cyan());
        println!();
    }

    fn command_dispatch(&mut self, input: &str) -> Result<(), String> {
        if input.is_empty() {
            return Ok(());
        }

        let command =
            scan_commands(input).ok_or_else(|| self.localization.get("invalid-command"))?;

        match command {
            Command::Exit => self.handle_exit(),
            Command::Help => self.display_help(),
            Command::Encrypt { text } => self.encrypt(&text)?,
            Command::Decrypt { text } => self.decrypt(&text)?,
            Command::History => self.history()?,
            Command::NewUser { name, password } => self.user_creation(&name, &password)?,
            Command::DeleteUser { name } => self.user_deletion(&name)?,
            Command::ListUsers => self.users_list(),
            Command::LoginUser { name, password } => self.user_login(&name, &password)?,
            Command::LogoutUser => self.user_logout()?,
            Command::NewSession { name } => self.session_creation(&name)?,
            Command::DeleteSession { name } => self.session_deletion(&name)?,
            Command::ListSessions => self.session_list()?,
            Command::OpenSession { name } => self.session_open(&name)?,
            Command::CloseSession => self.session_close()?,
            Command::Lang { language } => self.change_language(&language)?,
        }

        Ok(())
    }

    fn change_language(&mut self, language: &str) -> Result<(), String> {
        self.localization.set_locale(language)?;
        let lang_name = if language == "ru" || language == "ru-RU" {
            "Русский"
        } else if language == "en" || language == "en-US" {
            "English"
        } else {
            language
        };

        let args = fluent_args(&[("language", lang_name)]);
        info!(
            "{}",
            self.localization
                .get_with_args("language-changed", Some(&args))
        );
        println!();

        Ok(())
    }

    fn user_creation(&mut self, name: &str, password: &str) -> Result<(), String> {
        info!("{}", self.localization.get("creating-user"));
        self.user_manager.new_user(name, password)?;

        let args = fluent_args(&[("username", name)]);
        info!(
            "{}",
            self.localization.get_with_args("user-created", Some(&args))
        );

        self.user_manager.login(name, password)?;
        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn user_deletion(&mut self, name: &str) -> Result<(), String> {
        self.user_manager.delete_user(name)?;

        let args = fluent_args(&[("username", name)]);
        info!(
            "{}",
            self.localization.get_with_args("user-deleted", Some(&args))
        );

        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn users_list(&self) {
        self.display_section(self.localization.get("section-user-management"));
        let usernames = self.user_manager.get_usernames();

        if usernames.is_empty() {
            println!("{}", self.localization.get("no-users-found"));
        } else {
            usernames
                .iter()
                .enumerate()
                .for_each(|(i, name)| println!("  {}. {}", i + 1, name));
        }
        println!();
    }

    fn user_login(&mut self, name: &str, password: &str) -> Result<(), String> {
        self.user_manager.login(name, password)?;

        let args = fluent_args(&[("username", name)]);
        info!(
            "{}",
            self.localization
                .get_with_args("user-selected", Some(&args))
        );

        println!();

        Ok(())
    }

    fn user_logout(&mut self) -> Result<(), String> {
        self.user_manager.logout();
        info!("{}", self.localization.get("logged-out"));
        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn session_creation(&mut self, name: &str) -> Result<(), String> {
        self.display_section(self.localization.get("section-session-management"));

        let session_type = self.prompt_session_type()?;

        info!("{}", self.localization.get("creating-session"));
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or_else(|| self.localization.get("no-user-selected"))?;

        match session_type.as_str() {
            "in" => create_inbound_session(user, name, &self.localization)?,
            "out" => create_outbound_session(user, name, &self.localization)?,
            _ => return Err(self.localization.get("unknown-session-type")),
        }

        let args = fluent_args(&[("session_name", name)]);
        info!(
            "{}",
            self.localization
                .get_with_args("session-created", Some(&args))
        );

        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn session_deletion(&mut self, name: &str) -> Result<(), String> {
        self.user_manager.delete_session(name)?;

        let args = fluent_args(&[("session_name", name)]);
        info!(
            "{}",
            self.localization
                .get_with_args("session-deleted", Some(&args))
        );

        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn session_list(&mut self) -> Result<(), String> {
        self.display_section(self.localization.get("section-session-management"));

        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or_else(|| self.localization.get("no-user-selected"))?;

        let session_names = user.session_manager.get_session_names();

        if session_names.is_empty() {
            println!("{}", self.localization.get("no-sessions-found"));
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
            .ok_or_else(|| self.localization.get("no-user-selected"))?;

        user.session_manager.select_session(name)?;

        let args = fluent_args(&[("session_name", name)]);
        info!(
            "{}",
            self.localization
                .get_with_args("session-selected", Some(&args))
        );

        println!();

        Ok(())
    }

    fn session_close(&mut self) -> Result<(), String> {
        let user = self
            .user_manager
            .get_current_user_mut()
            .ok_or_else(|| self.localization.get("no-user-selected"))?;

        user.session_manager.deselect_session();
        info!("{}", self.localization.get("session-closed"));
        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn encrypt(&mut self, text: &str) -> Result<(), String> {
        println!();

        let encrypted = self.user_manager.encrypt(text)?;
        println!("{}", encrypted);

        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn decrypt(&mut self, text: &str) -> Result<(), String> {
        println!();

        let decrypted = self.user_manager.decrypt(text)?;
        println!("{}", decrypted);

        self.user_manager.autosave()?;
        println!();

        Ok(())
    }

    fn history(&self) -> Result<(), String> {
        self.display_section(self.localization.get("history-label"));

        let messages = self.user_manager.get_session_messages()?;
        println!("{}", messages);

        Ok(())
    }

    pub fn handle_exit(&mut self) {
        info!("{}", self.localization.get("exiting"));
        self.should_exit = true;
    }

    pub fn shutdown(self) -> Result<(), String> {
        info!("{}", self.localization.get("shutting-down"));
        self.user_manager.shutdown()
    }

    fn display_help(&self) {
        println!();
        println!(
            "{}",
            self.localization
                .get("help-available-commands")
                .yellow()
                .bold()
        );
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
        println!(
            "  {} - {}",
            "exit".cyan(),
            self.localization.get("help-exit")
        );
        println!(
            "  {} - {}",
            "help".cyan(),
            self.localization.get("help-help")
        );
        println!(
            "  {} - {}",
            "lang <en/ru>".cyan(),
            self.localization.get("help-lang")
        );
        println!();

        println!(
            "{}",
            self.localization
                .get("section-user-management")
                .yellow()
                .bold()
        );
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
        println!(
            "  {} - {}",
            "u new <username> <password>".cyan(),
            self.localization.get("help-user-new")
        );
        println!(
            "  {} - {}",
            "u delete <username>".cyan(),
            self.localization.get("help-user-delete")
        );
        println!(
            "  {} - {}",
            "u list".cyan(),
            self.localization.get("help-user-list")
        );
        println!(
            "  {} - {}",
            "u login <username> <password>".cyan(),
            self.localization.get("help-user-login")
        );
        println!(
            "  {} - {}",
            "u logout".cyan(),
            self.localization.get("help-user-logout")
        );
        println!();

        println!(
            "{}",
            self.localization
                .get("section-session-management")
                .yellow()
                .bold()
        );
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
        println!(
            "  {} - {}",
            "s new <session_name>".cyan(),
            self.localization.get("help-session-new")
        );
        println!(
            "  {} - {}",
            "s delete <session_name>".cyan(),
            self.localization.get("help-session-delete")
        );
        println!(
            "  {} - {}",
            "s list".cyan(),
            self.localization.get("help-session-list")
        );
        println!(
            "  {} - {}",
            "s open <session_name>".cyan(),
            self.localization.get("help-session-open")
        );
        println!(
            "  {} - {}",
            "s close".cyan(),
            self.localization.get("help-session-close")
        );
        println!();

        println!(
            "{}",
            self.localization.get("section-chatting").yellow().bold()
        );
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
        println!(
            "  {} - {}",
            "e <text>".cyan(),
            self.localization.get("help-encrypt")
        );
        println!(
            "  {} - {}",
            "d <text>".cyan(),
            self.localization.get("help-decrypt")
        );
        println!(
            "  {} - {}",
            "history".cyan(),
            self.localization.get("help-history")
        );
        println!();

        println!("{}", self.localization.get("help-tip-quotes").cyan());
    }

    fn display_section(&self, title: String) {
        println!();
        println!("{}", title.yellow().bold());
        println!("{}", "-".repeat(SECTION_WIDTH).grey());
    }

    fn prompt_session_type(&self) -> Result<String, String> {
        println!("{}", self.localization.get("session-type-title").grey());
        println!(
            "  {} ({})",
            "in".green(),
            self.localization.get("inbound-session")
        );
        println!(
            "  {} ({})",
            "out".green(),
            self.localization.get("outbound-session")
        );
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

fn create_inbound_session(
    user: &mut User,
    name: &str,
    localization: &Localization,
) -> Result<(), String> {
    println!("{}", localization.get("generating-keys").grey());
    let our_keys = user.session_manager.generate_keys(&mut user.account)?;
    println!("{}", localization.get("share-keys").grey());
    println!("{}", our_keys.bold());
    println!();

    println!("{}", localization.get("paste-other-keys").grey());
    let remote_keys = prompt_input();
    println!();

    println!("{}", localization.get("paste-init-message").grey());
    let first_message = prompt_input();
    println!();

    user.session_manager.establish_in_session(
        &mut user.account,
        name,
        &remote_keys,
        &first_message,
    )?;

    user.session_manager.select_session(name)
}

fn create_outbound_session(
    user: &mut User,
    name: &str,
    localization: &Localization,
) -> Result<(), String> {
    println!("{}", localization.get("generating-keys").grey());
    let our_keys = user.session_manager.generate_keys(&mut user.account)?;
    println!("{}", localization.get("share-keys").grey());
    println!("{}", our_keys.bold());
    println!();

    println!("{}", localization.get("paste-other-keys").grey());
    let remote_keys = prompt_input();
    println!();

    user.session_manager
        .establish_out_session(&mut user.account, name, &remote_keys)?;
    user.session_manager.select_session(name)?;

    println!("{}", localization.get("session-established").green());

    let init_message = user.session_manager.encrypt("")?;
    println!("{}", init_message.bold());

    Ok(())
}
